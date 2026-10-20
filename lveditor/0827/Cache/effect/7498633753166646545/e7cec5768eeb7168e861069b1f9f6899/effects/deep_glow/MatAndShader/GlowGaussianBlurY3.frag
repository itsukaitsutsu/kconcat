precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTextureY;

uniform float u_sampleY3;
uniform float u_sigmaY3;
uniform float u_dy3;
uniform float u_gamma;
uniform int u_borderType;
uniform int u_inverseGammaCorrection;
uniform int u_blurAlpha;

#define NO 0
#define YES 1
#define BORDER_TYPE_REPLICATE 1
#define BORDER_TYPE_BLACK 2
#define BORDER_TYPE_REFLECT 3
#define MAX_SAMPLE 1024

const float Eps = 1e-5;


float gaussianWeight(float x, float sigma)
{
    return exp(-0.5 * x * x / (sigma * sigma));
}


vec4 getColor(sampler2D tex, vec2 uv)
{
    vec4 color = texture2D(tex, uv);
    if (u_inverseGammaCorrection == 1) {
        // inverse gamma correction is not applied for alpha channel
        color.rgb = pow(color.rgb, vec3(u_gamma));
    }
    return color;
}


void main()
{
    if (u_sampleY3 < Eps) {
        gl_FragColor = texture2D(u_inputTextureY, uv0);
        return;
    }

    // init sumColor and sumWeight using current pixel
    vec4 oriColor = getColor(u_inputTextureY, uv0);
    float sumWeight = gaussianWeight(0., u_sigmaY3);
    vec4 sumColor = sumWeight * oriColor;

    // loop for both direction
    vec2 uv = uv0;
    for (int i = 1; i <= MAX_SAMPLE; i++) {
        float k = float(i);
        if (k > u_sampleY3) {
            break;
        }

        float y = k * u_dy3;
        float weight = gaussianWeight(y, u_sigmaY3);

        // borderType: Tranparent=0, Replicate=1, Black=2, Reflect=3
        // Transparent does NOT accumulate weight and color

        // left range
        uv.y = uv0.y - y;
        if (uv.y < 0.) {
            // uv out of border, process border conditions
            if (u_borderType == BORDER_TYPE_REPLICATE) {
                uv.y = 0.;
                sumColor += weight * getColor(u_inputTextureY, uv);
                sumWeight += weight;
            } else if (u_borderType == BORDER_TYPE_BLACK) {
                sumColor += weight * vec4(0., 0., 0., 1.);
                sumWeight += weight;
            } else if (u_borderType == BORDER_TYPE_REFLECT) {
                uv.y = -uv.y;
                sumColor += weight * getColor(u_inputTextureY, uv);
                sumWeight += weight;
            }
        } else {
            sumColor += weight * getColor(u_inputTextureY, uv);
            sumWeight += weight;
        }

        // right range
        uv.y = uv0.y + y;
        if (uv.y > 1.) {
            // uv out of border, process border conditions
            if (u_borderType == BORDER_TYPE_REPLICATE) {
                uv.y = 1.;
                sumColor += weight * getColor(u_inputTextureY, uv);
                sumWeight += weight;
            } else if (u_borderType == BORDER_TYPE_BLACK) {
                sumColor += weight * vec4(0., 0., 0., 1.);
                sumWeight += weight;
            } else if (u_borderType == BORDER_TYPE_REFLECT) {
                uv.y = 2. - uv.y;
                sumColor += weight * getColor(u_inputTextureY, uv);
                sumWeight += weight;
            }
        } else {
            sumColor += weight * getColor(u_inputTextureY, uv);
            sumWeight += weight;
        }
    }
    sumColor /= sumWeight;

    if (u_inverseGammaCorrection == YES) {
        // re-apply gamma correction for rgb
        sumColor.rgb = pow(sumColor.rgb, vec3(1. / u_gamma));
    }

    if (u_blurAlpha == NO) {
        sumColor.a = oriColor.a;
    }

    gl_FragColor = sumColor;
}
