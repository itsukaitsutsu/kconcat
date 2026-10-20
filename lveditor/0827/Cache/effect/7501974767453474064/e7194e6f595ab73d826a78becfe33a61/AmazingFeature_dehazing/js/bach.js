
const Amaz = effect.Amaz;

class InferenceEngine
{

    constructor()
    {
        this.engine = new Amaz.JSWrapByteNNEngine;
        this.engineConfig = new Amaz.JSWrapByteNNConfig;
    }

    loadModel(model)
    {
        this.engineConfig.modelBuffer = model.pData;
        this.engineConfig.modelBufferSize = model.length;
        // this.engineConfig.forwardType = 0;
		this.engineConfig.type = 0;
		this.engineConfig.numThread = 1;

        let errorCode = this.engine.Init(this.engineConfig);
        console.log(`[loadModel]: bytenn init code: ${errorCode}`);
    }

    destory() {
        this.engine.Release();
    }

    inference(tensors)
    {
        // let tensors_ = tensors;
        // let errorCode = this.engine.GetInputConfig(tensors_);
        // if (errorCode != 0) {
        //     console.error(`[Inference]: bytenn getInputConfig error ${errorCode}`)
        //     }
        let errorCode = this.engine.SetInput(tensors);
        if (errorCode != 0) {
        console.error(`[Inference]: bytenn setInput error ${errorCode}`)
        }
        errorCode = this.engine.Inference();
        if (errorCode != 0) {
        console.error(`[Inference]: bytenn Infernece error ${errorCode}`)
        }
        let outTensor = new Array();
        errorCode = this.engine.GetOutput(outTensor);
        if (errorCode != 0) {
        console.error(`[Inference]: bytenn GetOutput error ${errorCode}`)
        }
        console.log(`[Inference]: bytenn inference code: ${errorCode}`);
        return outTensor;
    }
}

exports.InferenceEngine = InferenceEngine;
exports.DefaultGraph = "test";

function getNodeIndex(nodeIndex)
{
    return nodeIndex.toString();
}

exports.getNodeIndex = getNodeIndex;

const ByteNNDataFormat = {
    NCHW : 0,
    NHWC : 1
}

const ByteNNDataType = {
    U8 : 0,
    Int8 : 1, //Support Int8 on CPU across Android/iOS/Mac/Windows and pixelbuffer on Mac and iOS
    Int16 : 2,
    Uint16 : 3,
    Float : 4,
    Fp16 : 5,
    Double : 6
}

const ByteNNErrorCode = {
    SUCCESS : 0,
    ERR_MEMORY_ALLOC : 1,
    NOT_IMPLEMENTED : 2,
    ERR_UNEXPECTED : 3,
    ERR_DATANOMATCH : 4,
    INPUT_DATA_ERROR : 5,
    CALL_BACK_STOP : 6,
    BACKEND_FALLBACK : 7,
    NULL_POINTER : 8,
    INVALID_POINTER : 9,
    INVALID_MODEL : 10,
    INFER_SIZE_ERROR : 11,
    NOT_SUPPORT : 12,
    DESTROYED_ERROR : 13,
    WRONG_LICENSE : 14,
    BROKEN_MODEL : 15,
    EARLY_STOP : 16,
}

const ByteNNForwardType = {
    CPU : 0,  // Android, iOS, Mac, Windows and Linux
    GPU : 1,  // Android, iOS, Mac, Windows
    DSP : 2,  // Android, iOS
    NPU : 3,  // Android
    Auto: 4, // Android, iOS, Mac, Windows and Linux

    METAL : 5,  // iOS
    OPENCL : 6, // Android, Mac, Windows
    OPENGL : 7,
    VULKAN : 8,
    CUDA : 9,   // Windows, Linux
    CoreML : 10, // iOS and Mac
}

exports.ByteNNDataFormat = ByteNNDataFormat;
exports.ByteNNDataType = ByteNNDataType;
exports.ByteNNErrorCode = ByteNNErrorCode;
exports.ByteNNForwardType = ByteNNForwardType;

const CvColorCode = {
    COLOR_BGR2BGR555 : 0x16,
    COLOR_BGR2BGR565 : 0xc,
    COLOR_BGR2BGRA : 0x0,
    COLOR_BGR2GRAY : 0x6,
    COLOR_BGR2HLS : 0x34,
    COLOR_BGR2HLS_FULL : 0x44,
    COLOR_BGR2HSV : 0x28,
    COLOR_BGR2HSV_FULL : 0x42,
    COLOR_BGR2LAB : 0x2c,
    COLOR_BGR2LUV : 0x32,
    COLOR_BGR2Lab : 0x2c,
    COLOR_BGR2Luv : 0x32,
    COLOR_BGR2RGB : 0x4,
    COLOR_BGR2RGBA : 0x2,
    COLOR_BGR2XYZ : 0x20,
    COLOR_BGR2YCR_CB : 0x24,
    COLOR_BGR2YCrCb : 0x24,
    COLOR_BGR2YUV : 0x52,
    COLOR_BGR2YUV_I420 : 0x80,
    COLOR_BGR2YUV_IYUV : 0x80,
    COLOR_BGR2YUV_YV12 : 0x84,
    COLOR_BGR5552BGR : 0x18,
    COLOR_BGR5552BGRA : 0x1c,
    COLOR_BGR5552GRAY : 0x1f,
    COLOR_BGR5552RGB : 0x19,
    COLOR_BGR5552RGBA : 0x1d,
    COLOR_BGR5652BGR : 0xe,
    COLOR_BGR5652BGRA : 0x12,
    COLOR_BGR5652GRAY : 0x15,
    COLOR_BGR5652RGB : 0xf,
    COLOR_BGR5652RGBA : 0x13,
    COLOR_BGRA2BGR : 0x1,
    COLOR_BGRA2BGR555 : 0x1a,
    COLOR_BGRA2BGR565 : 0x10,
    COLOR_BGRA2GRAY : 0xa,
    COLOR_BGRA2RGB : 0x3,
    COLOR_BGRA2RGBA : 0x5,
    COLOR_BGRA2YUV_I420 : 0x82,
    COLOR_BGRA2YUV_IYUV : 0x82,
    COLOR_BGRA2YUV_YV12 : 0x86,
    COLOR_BayerBG2BGR : 0x2e,
    COLOR_BayerBG2BGRA : 0x8b,
    COLOR_BayerBG2BGR_EA : 0x87,
    COLOR_BayerBG2BGR_VNG : 0x3e,
    COLOR_BayerBG2GRAY : 0x56,
    COLOR_BayerBG2RGB : 0x30,
    COLOR_BayerBG2RGBA : 0x8d,
    COLOR_BayerBG2RGB_EA : 0x89,
    COLOR_BayerBG2RGB_VNG : 0x40,
    COLOR_BayerGB2BGR : 0x2f,
    COLOR_BayerGB2BGRA : 0x8c,
    COLOR_BayerGB2BGR_EA : 0x88,
    COLOR_BayerGB2BGR_VNG : 0x3f,
    COLOR_BayerGB2GRAY : 0x57,
    COLOR_BayerGB2RGB : 0x31,
    COLOR_BayerGB2RGBA : 0x8e,
    COLOR_BayerGB2RGB_EA : 0x8a,
    COLOR_BayerGB2RGB_VNG : 0x41,
    COLOR_BayerGR2BGR : 0x31,
    COLOR_BayerGR2BGRA : 0x8e,
    COLOR_BayerGR2BGR_EA : 0x8a,
    COLOR_BayerGR2BGR_VNG : 0x41,
    COLOR_BayerGR2GRAY : 0x59,
    COLOR_BayerGR2RGB : 0x2f,
    COLOR_BayerGR2RGBA : 0x8c,
    COLOR_BayerGR2RGB_EA : 0x88,
    COLOR_BayerGR2RGB_VNG : 0x3f,
    COLOR_BayerRG2BGR : 0x30,
    COLOR_BayerRG2BGRA : 0x8d,
    COLOR_BayerRG2BGR_EA : 0x89,
    COLOR_BayerRG2BGR_VNG : 0x40,
    COLOR_BayerRG2GRAY : 0x58,
    COLOR_BayerRG2RGB : 0x2e,
    COLOR_BayerRG2RGBA : 0x8b,
    COLOR_BayerRG2RGB_EA : 0x87,
    COLOR_BayerRG2RGB_VNG : 0x3e,
    COLOR_COLORCVT_MAX : 0x8f,
    COLOR_GRAY2BGR : 0x8,
    COLOR_GRAY2BGR555 : 0x1e,
    COLOR_GRAY2BGR565 : 0x14,
    COLOR_GRAY2BGRA : 0x9,
    COLOR_GRAY2RGB : 0x8,
    COLOR_GRAY2RGBA : 0x9,
    COLOR_HLS2BGR : 0x3c,
    COLOR_HLS2BGR_FULL : 0x48,
    COLOR_HLS2RGB : 0x3d,
    COLOR_HLS2RGB_FULL : 0x49,
    COLOR_HSV2BGR : 0x36,
    COLOR_HSV2BGR_FULL : 0x46,
    COLOR_HSV2RGB : 0x37,
    COLOR_HSV2RGB_FULL : 0x47,
    COLOR_LAB2BGR : 0x38,
    COLOR_LAB2LBGR : 0x4e,
    COLOR_LAB2LRGB : 0x4f,
    COLOR_LAB2RGB : 0x39,
    COLOR_LBGR2LAB : 0x4a,
    COLOR_LBGR2LUV : 0x4c,
    COLOR_LBGR2Lab : 0x4a,
    COLOR_LBGR2Luv : 0x4c,
    COLOR_LRGB2LAB : 0x4b,
    COLOR_LRGB2LUV : 0x4d,
    COLOR_LRGB2Lab : 0x4b,
    COLOR_LRGB2Luv : 0x4d,
    COLOR_LUV2BGR : 0x3a,
    COLOR_LUV2LBGR : 0x50,
    COLOR_LUV2LRGB : 0x51,
    COLOR_LUV2RGB : 0x3b,
    COLOR_Lab2BGR : 0x38,
    COLOR_Lab2LBGR : 0x4e,
    COLOR_Lab2LRGB : 0x4f,
    COLOR_Lab2RGB : 0x39,
    COLOR_Luv2BGR : 0x3a,
    COLOR_Luv2LBGR : 0x50,
    COLOR_Luv2LRGB : 0x51,
    COLOR_Luv2RGB : 0x3b,
    COLOR_M_RGBA2RGBA : 0x7e,
    COLOR_RGB2BGR : 0x4,
    COLOR_RGB2BGR555 : 0x17,
    COLOR_RGB2BGR565 : 0xd,
    COLOR_RGB2BGRA : 0x2,
    COLOR_RGB2GRAY : 0x7,
    COLOR_RGB2HLS : 0x35,
    COLOR_RGB2HLS_FULL : 0x45,
    COLOR_RGB2HSV : 0x29,
    COLOR_RGB2HSV_FULL : 0x43,
    COLOR_RGB2LAB : 0x2d,
    COLOR_RGB2LUV : 0x33,
    COLOR_RGB2Lab : 0x2d,
    COLOR_RGB2Luv : 0x33,
    COLOR_RGB2RGBA : 0x0,
    COLOR_RGB2XYZ : 0x21,
    COLOR_RGB2YCR_CB : 0x25,
    COLOR_RGB2YCrCb : 0x25,
    COLOR_RGB2YUV : 0x53,
    COLOR_RGB2YUV_I420 : 0x7f,
    COLOR_RGB2YUV_IYUV : 0x7f,
    COLOR_RGB2YUV_YV12 : 0x83,
    COLOR_RGBA2BGR : 0x3,
    COLOR_RGBA2BGR555 : 0x1b,
    COLOR_RGBA2BGR565 : 0x11,
    COLOR_RGBA2BGRA : 0x5,
    COLOR_RGBA2GRAY : 0xb,
    COLOR_RGBA2M_RGBA : 0x7d,
    COLOR_RGBA2RGB : 0x1,
    COLOR_RGBA2YUV_I420 : 0x81,
    COLOR_RGBA2YUV_IYUV : 0x81,
    COLOR_RGBA2YUV_YV12 : 0x85,
    COLOR_RGBA2mRGBA : 0x7d,
    COLOR_XYZ2BGR : 0x22,
    COLOR_XYZ2RGB : 0x23,
    COLOR_YCR_CB2BGR : 0x26,
    COLOR_YCR_CB2RGB : 0x27,
    COLOR_YCrCb2BGR : 0x26,
    COLOR_YCrCb2RGB : 0x27,
    COLOR_YUV2BGR : 0x54,
    COLOR_YUV2BGRA_I420 : 0x69,
    COLOR_YUV2BGRA_IYUV : 0x69,
    COLOR_YUV2BGRA_NV12 : 0x5f,
    COLOR_YUV2BGRA_NV21 : 0x61,
    COLOR_YUV2BGRA_UYNV : 0x70,
    COLOR_YUV2BGRA_UYVY : 0x70,
    COLOR_YUV2BGRA_Y422 : 0x70,
    COLOR_YUV2BGRA_YUNV : 0x78,
    COLOR_YUV2BGRA_YUY2 : 0x78,
    COLOR_YUV2BGRA_YUYV : 0x78,
    COLOR_YUV2BGRA_YV12 : 0x67,
    COLOR_YUV2BGRA_YVYU : 0x7a,
    COLOR_YUV2BGR_I420 : 0x65,
    COLOR_YUV2BGR_IYUV : 0x65,
    COLOR_YUV2BGR_NV12 : 0x5b,
    COLOR_YUV2BGR_NV21 : 0x5d,
    COLOR_YUV2BGR_UYNV : 0x6c,
    COLOR_YUV2BGR_UYVY : 0x6c,
    COLOR_YUV2BGR_Y422 : 0x6c,
    COLOR_YUV2BGR_YUNV : 0x74,
    COLOR_YUV2BGR_YUY2 : 0x74,
    COLOR_YUV2BGR_YUYV : 0x74,
    COLOR_YUV2BGR_YV12 : 0x63,
    COLOR_YUV2BGR_YVYU : 0x76,
    COLOR_YUV2GRAY_420 : 0x6a,
    COLOR_YUV2GRAY_I420 : 0x6a,
    COLOR_YUV2GRAY_IYUV : 0x6a,
    COLOR_YUV2GRAY_NV12 : 0x6a,
    COLOR_YUV2GRAY_NV21 : 0x6a,
    COLOR_YUV2GRAY_UYNV : 0x7b,
    COLOR_YUV2GRAY_UYVY : 0x7b,
    COLOR_YUV2GRAY_Y422 : 0x7b,
    COLOR_YUV2GRAY_YUNV : 0x7c,
    COLOR_YUV2GRAY_YUY2 : 0x7c,
    COLOR_YUV2GRAY_YUYV : 0x7c,
    COLOR_YUV2GRAY_YV12 : 0x6a,
    COLOR_YUV2GRAY_YVYU : 0x7c,
    COLOR_YUV2RGB : 0x55,
    COLOR_YUV2RGBA_I420 : 0x68,
    COLOR_YUV2RGBA_IYUV : 0x68,
    COLOR_YUV2RGBA_NV12 : 0x5e,
    COLOR_YUV2RGBA_NV21 : 0x60,
    COLOR_YUV2RGBA_UYNV : 0x6f,
    COLOR_YUV2RGBA_UYVY : 0x6f,
    COLOR_YUV2RGBA_Y422 : 0x6f,
    COLOR_YUV2RGBA_YUNV : 0x77,
    COLOR_YUV2RGBA_YUY2 : 0x77,
    COLOR_YUV2RGBA_YUYV : 0x77,
    COLOR_YUV2RGBA_YV12 : 0x66,
    COLOR_YUV2RGBA_YVYU : 0x79,
    COLOR_YUV2RGB_I420 : 0x64,
    COLOR_YUV2RGB_IYUV : 0x64,
    COLOR_YUV2RGB_NV12 : 0x5a,
    COLOR_YUV2RGB_NV21 : 0x5c,
    COLOR_YUV2RGB_UYNV : 0x6b,
    COLOR_YUV2RGB_UYVY : 0x6b,
    COLOR_YUV2RGB_Y422 : 0x6b,
    COLOR_YUV2RGB_YUNV : 0x73,
    COLOR_YUV2RGB_YUY2 : 0x73,
    COLOR_YUV2RGB_YUYV : 0x73,
    COLOR_YUV2RGB_YV12 : 0x62,
    COLOR_YUV2RGB_YVYU : 0x75,
    COLOR_YUV420P2BGR : 0x63,
    COLOR_YUV420P2BGRA : 0x67,
    COLOR_YUV420P2GRAY : 0x6a,
    COLOR_YUV420P2RGB : 0x62,
    COLOR_YUV420P2RGBA : 0x66,
    COLOR_YUV420SP2BGR : 0x5d,
    COLOR_YUV420SP2BGRA : 0x61,
    COLOR_YUV420SP2GRAY : 0x6a,
    COLOR_YUV420SP2RGB : 0x5c,
    COLOR_YUV420SP2RGBA : 0x60,
    COLOR_YUV420p2BGR : 0x63,
    COLOR_YUV420p2BGRA : 0x67,
    COLOR_YUV420p2GRAY : 0x6a,
    COLOR_YUV420p2RGB : 0x62,
    COLOR_YUV420p2RGBA : 0x66,
    COLOR_YUV420sp2BGR : 0x5d,
    COLOR_YUV420sp2BGRA : 0x61,
    COLOR_YUV420sp2GRAY : 0x6a,
    COLOR_YUV420sp2RGB : 0x5c,
    COLOR_YUV420sp2RGBA : 0x60,
    COLOR_mRGBA2RGBA : 0x7e,
    CONTOURS_MATCH_I1 : 0x1,
    CONTOURS_MATCH_I2 : 0x2,
    CONTOURS_MATCH_I3 : 0x3,
    COVAR_COLS : 0x10,
    COVAR_NORMAL : 0x1,
    COVAR_ROWS : 0x8,
    COVAR_SCALE : 0x4,
    COVAR_SCRAMBLED : 0x0,
    COVAR_USE_AVG : 0x2,
    CV_16S : 0x3,
    CV_16SC1 : 0x3,
    CV_16SC2 : 0xb,
    CV_16SC3 : 0x13,
    CV_16SC4 : 0x1b,
    CV_16U : 0x2,
    CV_16UC1 : 0x2,
    CV_16UC2 : 0xa,
    CV_16UC3 : 0x12,
    CV_16UC4 : 0x1a,
    CV_32F : 0x5,
    CV_32FC1 : 0x5,
    CV_32FC2 : 0xd,
    CV_32FC3 : 0x15,
    CV_32FC4 : 0x1d,
    CV_32S : 0x4,
    CV_32SC1 : 0x4,
    CV_32SC2 : 0xc,
    CV_32SC3 : 0x14,
    CV_32SC4 : 0x1c,
    CV_64F : 0x6,
    CV_64FC1 : 0x6,
    CV_64FC2 : 0xe,
    CV_64FC3 : 0x16,
    CV_64FC4 : 0x1e,
    CV_8S : 0x1,
    CV_8SC1 : 0x1,
    CV_8SC2 : 0x9,
    CV_8SC3 : 0x11,
    CV_8SC4 : 0x19,
    CV_8U : 0x0,
    CV_8UC1 : 0x0,
    CV_8UC2 : 0x8,
    CV_8UC3 : 0x10,
    CV_8UC4 : 0x18
}

const CvCOMOP = {
    CMP_EQ: 0, 
    CMP_GT: 1, 
    CMP_GE: 2, 
    CMP_LT: 3, 
    CMP_LE: 4,
    CMP_NE: 5,
}

exports.CvCOMOP = CvCOMOP;
exports.CvColorCode = CvColorCode;
