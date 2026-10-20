precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform sampler2D u_darkChannelTex;
uniform sampler2D u_airLightTex;
uniform float u_dehazeParam;

float getAdjustWeight(float darkColor, float dehazeParam)
{
    float weightDark = pow(2. * (darkColor - 0.5), 2.);
    float weight = clamp(weightDark, 0., 1.);

    if (dehazeParam < 0.5) {
        dehazeParam = 2. * dehazeParam; // normalize to [0., 1.]
        weight = mix(1., weight, dehazeParam);
    }
    return weight;
}

void main()
{
    // get infos from textures
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float darkColor = texture2D(u_darkChannelTex, uv0).r;
    float airLight = texture2D(u_airLightTex, vec2(0.5, 0.5)).r; // airLight is a single number (tex size is 1x1)

    // dehaze
    float dehazeWeight = clamp(u_dehazeParam, 0.5, 0.95);
    float trans = 1. - dehazeWeight * darkColor;
    vec4 resColor = oriColor;
    resColor.rgb = (oriColor.rgb - airLight) / max(trans, 0.1) + airLight;
    float adjustWeight = getAdjustWeight(darkColor, u_dehazeParam); // to adjust dehaze strengh
    resColor = mix(resColor, oriColor, adjustWeight);
    gl_FragColor = resColor;
}
