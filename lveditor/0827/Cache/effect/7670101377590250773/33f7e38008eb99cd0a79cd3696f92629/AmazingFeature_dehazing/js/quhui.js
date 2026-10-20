'use strict';
const Amaz = effect.Amaz;
const cv = Amaz.JSWrapCV;
const { DefaultGraph, getNodeIndex, CvColorCode, CvCOMOP } = require("./bach.js");

class quhui {
  constructor(params, systemParams, alg) {
    this.first = true
    this.params = params;
    this.alg = alg;
    this.systemParams = systemParams;
    this.width = 0;
    this.height = 0;
    this.lastBlitImage = null;
    this.strength = 0;
    this.paramChangeCallbackMap = new Map([
      ['script_path', (value) => { this.params.script_path = value; }]]);

    const DegrayProcessor = {
      // find the darkest channel
      _dark_channel: function (im, sz) {
        const bgr = Array();
        cv.split(im, bgr);
        let b = bgr[0];
        let g = bgr[1];
        let r = bgr[2];

        let dc = cv.Mat();
        cv.min(r, g, dc); // dc = min(r, g)
        cv.min(dc, b, dc); // dc = min(dc, b) == min(r, g, b)

        let kernel = cv.getStructuringElement(0, [sz, sz]);
        let dark = cv.Mat(dc.rows, dc.cols, dc.type);
        cv.erode(dc, dark, kernel);

        return dark; // Return CV_8U or CV_32F Mat, same type as input channels
      },

      _transmission_estimate: function (img_rgb_32f, atmosphere, useSize = false) {
        const omega = 0.95;
        let dark = null;
        let img_normalized = null;
        let transmission = cv.Mat();
        let omegaDark = null;
        let atm_scalar = null;
        let ones = null;
        let omegaMat = null;
        let sz = 0;
        const one = [1];

        if (!useSize) {
          let min_dim = Math.min(img_rgb_32f.rows, img_rgb_32f.cols);
          sz = Math.round(1.5 * min_dim / 40);
          sz = Math.max(sz, 1); // Ensure sz is at least 1
        }
        else sz = 15
        // console.log("size", sz);
        atm_scalar = cv.Mat(img_rgb_32f.rows, img_rgb_32f.cols, img_rgb_32f.type, atmosphere);
        img_rgb_32f.div(atm_scalar);
        img_normalized = img_rgb_32f;

        dark = this._dark_channel(img_normalized, sz); // Assuming darkChannel works with CV_32F

        // // transmission = 1 - omega * dark;
        ones = cv.Mat(dark.rows, dark.cols, dark.type, one);
        omegaMat = cv.Mat(dark.rows, dark.cols, dark.type, [omega]);

        omegaDark = dark.mul(omegaMat);
        cv.subtract(ones, omegaDark, transmission);

        return { te: transmission, dark_div_A: dark };
      },

      _transmission_refine: function (im, et) {
        // im: input image Mat (CV_8UC3)
        // et: estimated transmission Mat (e.g., CV_64F)
        let gray = cv.Mat();
        let gray_f64 = cv.Mat();
        let t = null;

        cv.cvtColor(im, gray, CvColorCode.COLOR_RGB2GRAY); // Assuming BGR input for consistency
        gray.convertTo(gray_f64, CvColorCode.CV_64F, 1.0 / 255.0);

        let r = 60; // radius
        let eps = 0.0001;
        t = this._guided_filter(gray_f64, et, r, 1, eps);

        return t; // Return refined transmission (CV_64F)
      },

      _guided_filter: function (im, p, r, s, eps) {
        // im: guidance image Mat (e.g., CV_64F)
        // p: filtering input Mat (e.g., CV_64F)
        // r: radius
        // eps: epsilon
        let orginWH = [im.cols, im.rows];
        let im_sub = cv.Mat();
        cv.resize(im, im_sub, [Math.floor(im.cols / s), Math.floor(im.rows / s)])
        cv.resize(p, p, [Math.floor(p.cols / s), Math.floor(p.rows / s)])
        let mean_I = cv.Mat();
        let mean_p = cv.Mat();
        let mean_Ip = cv.Mat();
        let mean_II = cv.Mat();
        let mean_I_p = cv.Mat();
        let ksize = [Math.floor(r / s), Math.floor(r / s)];
        let Ip = null;
        let II = null;
        let cov_Ip = cv.Mat();
        let var_I = cv.Mat();
        let mean_I_mean_I = cv.Mat();
        let a = cv.Mat();
        let a_mean_I = cv.Mat();
        let b = cv.Mat();
        let mean_a = cv.Mat();
        let mean_b = cv.Mat();
        let mean_a_im = null;

        cv.blur(im_sub, mean_I, ksize);
        // return mean_I;
        cv.blur(p, mean_p, ksize);

        Ip = im_sub.mul(p);
        cv.blur(Ip, mean_Ip, ksize);

        II = im_sub.mul(im_sub); // II = im * im
        cv.blur(II, mean_II, ksize);
        mean_I_p = mean_I.mul(mean_p);
        // cov_Ip = mean_Ip - mean_I * mean_p;
        cv.subtract(mean_Ip, mean_I_p, cov_Ip);
        // var_I = mean_II - mean_I * mean_I;
        mean_I_mean_I = mean_I.mul(mean_I);
        cv.subtract(mean_II, mean_I_mean_I, var_I);
        // a = cov_Ip / (var_I + eps);
        var_I.add(cv.Mat(var_I.rows, var_I.cols, var_I.type, [eps]));
        cov_Ip.div(var_I);

        // b = mean_p - a * mean_I;
        a = cov_Ip;
        a_mean_I = a.mul(mean_I);
        mean_p.sub(a_mean_I);
        b = mean_p;

        // mean_a = cv.blur(a, cv.CV_64F, ksize);
        // mean_b = cv.blur(b, cv.CV_64F, ksize);
        cv.blur(a, mean_a, ksize);
        cv.blur(b, mean_b, ksize);

        cv.resize(mean_a, mean_a, orginWH);
        cv.resize(mean_b, mean_b, orginWH);
        // q = mean_a * im + mean_b;
        mean_a_im = mean_a.mul(im);
        mean_a_im.add(mean_b);

        return mean_a_im; // Return CV_64F Mat
      },

      _atmLight: function (im, dark) {
        // im: input image Mat (e.g., CV_64FC3)
        // dark: dark channel Mat (e.g., CV_64F)
        let h = im.rows;
        let w = im.cols;
        let imsz = h * w;
        let darkVec = []; // Array of {value, index}
        const darkArr = new Float64Array(dark.data);
        let indices = -1;
        let numpx = Math.trunc(Math.max(Math.floor(imsz / 1000), 1));
        let atmsum = [0, 0, 0];
        let A = null;

        for (let i = 0; i < imsz; i++) {
          darkVec.push({ value: darkArr[i], index: i });
        }

        darkVec.sort((a, b) => a.value - b.value); // Sort ascending by value
        indices = darkVec.slice(imsz - numpx).map(item => item.index); // Get indices of top numpx values
        // console.log(indices.length);
        const viewIm = new Float64Array(im.data);
        for (let i = 0; i < indices.length; i++) {
          let idx = indices[i];
          atmsum[0] += viewIm[idx * 3 + 0];
          atmsum[1] += viewIm[idx * 3 + 1];
          atmsum[2] += viewIm[idx * 3 + 2];
        }
        A = [atmsum[0] / numpx, atmsum[1] / numpx, atmsum[2] / numpx]
        return A;
      },

      _precompute_degray: function (img_rgb_32f, atmosphere) {
        let gray = cv.Mat();
        const { te, dark_div_A } = this._transmission_estimate(img_rgb_32f, atmosphere); // te, dark are CV_32F
        cv.cvtColor(img_rgb_32f, gray, CvColorCode.COLOR_RGB2GRAY); // gray is CV_32F if input is

        let min_size = Math.min(img_rgb_32f.rows, img_rgb_32f.cols);

        let radius = 8 * Math.round(1.5 * min_size / 40);
        radius = Math.max(radius, 1);

        // Ensure types are CV_32F for guided filter
        let gray_f32 = cv.Mat();
        if (gray.type !== CvColorCode.CV_32F) {
          gray.convertTo(gray_f32, CvColorCode.CV_32F);
        } else {
          gray_f32 = gray; // Use directly if already CV_32F
        }
        
        let transmission_refined = this._guided_filter(
          gray_f32,
          te, // te should be CV_32F
          radius, // radius for guided filter
          1,
          1e-4
        ); // transmission_refined is CV_32F

        let trans = cv.Mat();
        transmission_refined.convertTo(trans, CvColorCode.CV_8UC1, 255);
        return { transmission: trans, dark: dark_div_A }; // Return object, both CV_32F
      },

      _get_skin_mask: function (img_resized_rgb, segInfo) {
        if (segInfo) {
          const originSkinMask = cv.Mat(segInfo.data);
          let temp = cv.Mat();
          cv.resize(originSkinMask, temp, [img_resized_rgb.cols, img_resized_rgb.rows]);
          this.skin_mask = temp;
          return temp;
        }
        return null;
      },

      apply: function (image_uint8, image_source_uint8, atmosphere) {
        let alpha_s_full = 0;
        let image_32f = cv.Mat();
        let image_source_32f = cv.Mat();
        let _human = 0.0;
        let tv = 0.8; // Default value
        image_uint8.convertTo(image_32f, CvColorCode.CV_32F, 1.0 / 255.0);
        image_source_uint8.convertTo(image_source_32f, CvColorCode.CV_32F, 1.0 / 255.0);
        const { transmission, dark } = this._precompute_degray(image_32f, [atmosphere, atmosphere, atmosphere]); // Both CV_32F

        if (this.skin_mask) {
          let skin_mask_f32 = cv.Mat();
          this.skin_mask.convertTo(skin_mask_f32, CvColorCode.CV_32F, 1.0 / 255.0);

          let bin_mask = cv.Mat();
          cv.threshold(skin_mask_f32, bin_mask, 0.7, 1.0, 0); // Threshold 0.7 (normalized)

          let area_skin = cv.sum(bin_mask);

          // 定义平滑处理的系数，可根据实际情况调整
          const SMOOTHING_FACTOR = 0.1; 
          // 如果是第一次处理，初始化 _human 为 0
          if (!this.hasOwnProperty('_smoothedHuman')) {
              this._smoothedHuman = 0; 
          }

          // 根据 area_skin[0] 的变化对 _human 进行平滑处理
          if (area_skin[0] > 10) {
              const targetHuman = 1.0;
              //this._smoothedHuman += SMOOTHING_FACTOR * (targetHuman - this._smoothedHuman);
              this._smoothedHuman = targetHuman;
          } else {
              const targetHuman = 0.0;
              //this._smoothedHuman += SMOOTHING_FACTOR * (targetHuman - this._smoothedHuman);
              this._smoothedHuman = targetHuman;
          }

          // 确保 _smoothedHuman 在 [0, 1] 范围内
          this._smoothedHuman = Math.max(0, Math.min(1, this._smoothedHuman));
          // 将平滑后的 _human 值赋值给最终使用的 _human 变量
          _human = this._smoothedHuman;
          // 使用 _human 作为参数在 0 和 0.25 之间进行线性插值
          // _human = 1.0;
          alpha_s_full = 0.25 * _human;
          // if (area_skin[0] > 10) {
          //   // console.log("skin calculate!");
          //   _human = 1.0;
          //   // Resize dark channel to mask size if needed
          //   let dark_resized = null;
          //   if (dark.rows !== bin_mask.rows || dark.cols !== bin_mask.cols) {
          //     dark_resized = cv.Mat();
          //     cv.resize(dark, dark_resized, [bin_mask.cols, bin_mask.rows]);
          //   } else {
          //     dark_resized = dark.deepCopy();
          //   }

          //   // dsm = dark * bin_mask
          //   let dsm = null;
          //   dsm = dark_resized.mul(bin_mask); // Element-wise multiply

          //   let sum_dsm = cv.sum(dsm); // Returns array [sum]
          //   let area_sum = sum_dsm[0] / area_skin[0];

          //   tv = 0.8 - 0.2 * (area_sum - 0.24) / 0.14;
          //   tv = Math.max(0.6, Math.min(tv, 0.9)); // Clip tv

          //   let sat_masked = null;
          //   sat_masked = bin_mask.mul(cv.Mat(bin_mask.rows, bin_mask.cols, bin_mask.type, [area_sum])); // Mask saturation
          //   let sum_sat = cv.sum(sat_masked);
          //   let s_avg = sum_sat[0] / area_skin[0];
          //   alpha_s_full = ((s_avg - 0.29) / 0.14) * 0.2 + 0.1
          //   // console.log(alpha_s_full);
          //   let up_alpha = true ? 0.25 : 0.18;
          //   alpha_s_full = up_alpha;
          // } else {
          //   _human = 0.0;
          // }
        }
        return { transmission: transmission, human: _human, tv: tv, alpha_skin: alpha_s_full};
      }
    }

    this.DegrayProcessor = DegrayProcessor;
  }

  getResizedImage(input_img) {
    let h = input_img.rows;
    let w = input_img.cols;
    let sz = [w, h];

    let min_bs = Math.min(480, Math.min(sz[0], sz[1]));
    min_bs = Math.trunc(Math.ceil(min_bs / 16) * 16 + 0.5);

    let expect_width, expect_height;
    if (sz[0] < sz[1]) {
      expect_width = min_bs;
      expect_height = Math.trunc((Math.round(sz[1] * min_bs / sz[0]) + 15) / 16) * 16;
    } else {
      expect_height = min_bs;
      expect_width = Math.trunc((Math.round(sz[0] * min_bs / sz[1]) + 15) / 16) * 16;
    }

    let resized_img = cv.Mat();
    cv.resize(input_img, resized_img, [expect_width, expect_height]);
    // console.log(`expected_width: ${expect_width}, expected_height ${expect_height}`);
    return resized_img;
  }

  calculateDehazeStrength(src) {
    // src: input image Mat (CV_8UC3)
    let I = cv.Mat();
    let dark = null;
    let A = null;
    let originTest = cv.Mat();
    let refinedTrans = null;
    let mask = cv.Mat();
    const threshold = [0.8];

    src.convertTo(I, CvColorCode.CV_64F, 1.0 / 255.0);
    dark = this.DegrayProcessor._dark_channel(I, 15); // CV_64F
    A = this.DegrayProcessor._atmLight(I, dark);
    // console.log("Atmosphere", A[0], A[1], A[2])

    const { te, dark_div_A } = this.DegrayProcessor._transmission_estimate(I, A, true);

    // Note: TransmissionRefine expects CV_8UC3 input 'im'
    // TODO:🔞 This function need validation
    refinedTrans = this.DegrayProcessor._transmission_refine(src, te);

    cv.compare(refinedTrans, threshold, mask, CvCOMOP.CMP_LE); // mask is CV_8U
    refinedTrans.convertTo(originTest, CvColorCode.CV_8UC1, 255.0);

    // Calculate mean of dark channel where t <= threshold
    // Use mean() with mask. Ensure types match or convert mask.
    // mean() returns Scalar. Need to extract the value.
    let meanResult = cv.mean(dark, mask); // Calculates mean over non-zero mask pixels
    let dark_mean = meanResult[0]; // Mean is calculated for each channel, but dark is single channel
    // Clip the strength
    let dehaze_strength = Math.max(0.0, Math.min(dark_mean, 1.0));
    return dehaze_strength;
  }

  storeCurveTexture() {
    const _low_curve23 = [
      0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,2,2,2,2,2,3,3,3,4,4,5,5,6,7,7,8,9,11,12,13,15,17,19,21,23,26,29,32,36,39,43,48,53,58,63,69,75,81,88,95,103,111,119,128,137,146,155,164,174,184,193,204,214,224,234,245,255
    ];
    const _low_curve = [
      0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,5,5,5,5,5,5,5,5,5,6,6,6,6,6,6,7,7,7,7,7,7,8,8,8,8,9,9,9,9,10,10,10,10,11,11,11,12,12,12,13,13,14,14,15,15,15,16,16,17,18,18,19,19,20,20,21,22,23,23,24,25,26,26,27,28,29,30,31,32,33,34,35,36,38,39,40,41,43,44,46,47,49,50,52,54,55,57,59,61,63,66,68,71,73,76,79,82,85,88,92,95,99,104,108,114,119,125,132,140,149,159,173,190,215,255
    ];
    const _up_curve1 = [
      0,2,4,6,8,10,12,14,17,19,21,23,25,27,29,31,33,35,37,39,41,43,45,47,49,51,53,55,57,59,61,63,65,67,68,70,72,74,76,78,80,82,83,85,87,89,91,92,94,96,98,99,101,103,104,106,108,110,111,113,114,116,118,119,121,123,124,126,127,129,130,132,133,135,136,138,139,141,142,144,145,146,148,149,151,152,153,155,156,157,159,160,161,163,164,165,166,168,169,170,171,173,174,175,176,177,179,180,181,182,183,184,185,186,188,189,190,191,192,193,194,195,196,197,198,199,200,201,201,202,203,204,205,206,207,208,208,209,210,211,212,212,213,214,214,215,216,217,217,218,219,219,220,220,221,222,222,223,223,224,225,225,226,226,227,227,228,228,229,229,230,230,230,231,231,232,232,233,233,233,234,234,235,235,235,236,236,237,237,237,238,238,238,239,239,239,240,240,240,241,241,241,242,242,242,243,243,243,243,244,244,244,245,245,245,245,246,246,246,246,247,247,247,248,248,248,248,249,249,249,249,249,250,250,250,250,251,251,251,251,252,252,252,252,253,253,253,253,253,254,254,254,254,255,255,255
    ];
    const low_curve23Mat = cv.Mat(1, _low_curve23.length, CvColorCode.CV_8UC1, _low_curve23);
    const low_curveMat = cv.Mat(1, _low_curve.length, CvColorCode.CV_8UC1, _low_curve);
    const up_curveMat = cv.Mat(1, _up_curve1.length, CvColorCode.CV_8UC1, _up_curve1);
    const all = [low_curve23Mat, low_curveMat, up_curveMat];
    let result = cv.Mat();
    cv.merge(all, result);
    return result;
  }

  // input: type effect.Amaz.AEAlgorithmResult
  execute(input, output) {
    // origin image type is CV_8UC4
    const currentBlitImage = input.getBlitImage(DefaultGraph, getNodeIndex(0));
    const image_rgba = cv.Mat(currentBlitImage);
    let img_resized_rgba = this.getResizedImage(image_rgba);
    let recalulateStrength = true;
    // whether this image need recalculate strength
    if(this.lastBlitImage == null || this.width != img_resized_rgba.cols || this.height != img_resized_rgba.rows) {
      this.lastBlitImage = img_resized_rgba.deepCopy();
      this.width = this.lastBlitImage.cols;
      this.height = this.lastBlitImage.rows;
    } else {
      const lastImage_rgba = this.lastBlitImage;
      let lastImage_gray = cv.Mat();
      let image_gray = cv.Mat();
      cv.cvtColor(lastImage_rgba, lastImage_gray, CvColorCode.COLOR_RGBA2GRAY);
      cv.cvtColor(img_resized_rgba, image_gray, CvColorCode.COLOR_RGBA2GRAY);
      let a = cv.Mat();
      cv.subtract(image_gray, lastImage_gray, a);
      const aa = a.mul(a);
      const mse = cv.mean(aa);
      recalulateStrength = mse[0] > 100;
      this.lastBlitImage = img_resized_rgba.deepCopy();
    }

    let skin_mask = null;
    let image_rgb = cv.Mat();
    
    let img_resized_rgb = cv.Mat();
    cv.cvtColor(image_rgba, image_rgb, CvColorCode.COLOR_RGBA2RGB);
    cv.cvtColor(img_resized_rgba, img_resized_rgb, CvColorCode.COLOR_RGBA2RGB);
    
    if (recalulateStrength) {
      this.strength = this.calculateDehazeStrength(img_resized_rgb);
    }
    
    const maskData = this.DegrayProcessor._get_skin_mask(img_resized_rgb, input.getSkinSegInfo());
    if(maskData && maskData.rows > 0) {
      skin_mask = maskData.toImage(Amaz.PixelFormat.L8Unorm, true);
    }
    const { transmission, human, tv, alpha_skin } = this.DegrayProcessor.apply(img_resized_rgb, image_rgb, 0.8, 0.25);
    let resized_trans = cv.Mat();
    cv.resize(transmission, resized_trans, [image_rgba.cols, image_rgba.rows]);
    let trans = resized_trans.toImage(Amaz.PixelFormat.L8Unorm, true);


    if(this.first) {
      const curves = this.storeCurveTexture();
      output.set("curves", curves.toImage(Amaz.PixelFormat.RGBA8Unorm, true));
      this.first = false;
    }
    output.set("human", human);
    output.set("tv", tv);
    output.set("transmission", trans);
    output.set("skin_mask", skin_mask);
    output.set("alpha_skin", alpha_skin);
    output.set("strength", this.strength);
  }

  destory() {

  }
}

exports.quhui = quhui;