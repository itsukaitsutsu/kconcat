precision highp float;
varying highp vec2 uv0;
uniform sampler2D pyrLastTex;
uniform sampler2D pyrCurTex;
uniform float subFlag;
uniform float fact;
uniform float sigma;

vec4 enCode(float a){
    float a1 = floor(a*255.);
    float b1 = fract(a*255.);
    float a2 = floor(b1*255.);
    float b2 = fract(b1*255.);
    float a3 = floor(b2*255.);
    float b3 = fract(b2*255.);
    return vec4(a1/255.,a2/255.,a3/255.,b3);
}
float deCode(vec4 a){
    return a.x+a.y/255.+a.z/(255.*255.)+a.w/(255.*255.*255.);
}

void main()
{
    vec4 lastColor = texture2D(pyrLastTex, uv0);
    vec4 curColor = texture2D(pyrCurTex, uv0);
    // float curValue = deCode(curColor);
    float lastValue = (lastColor.r);
    vec4 resultColor = vec4(0.);// lastColor;
    float diff = lastValue;
    // diff = diff + fact * diff * exp(-diff * diff / (2. * sigma * sigma));
    // diff = (diff + 1.) / 2.;

    resultColor = vec4(diff);
    // resultColor = enCode(diff);
    resultColor.a = 1.;
    gl_FragColor = resultColor;
}
