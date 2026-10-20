precision highp float;
varying highp vec2 uv0;
uniform sampler2D pyrLastTex;
uniform sampler2D pyrCurTex;
uniform float subFlag;
uniform float fact;
uniform float resOffset;

vec4 enCode(float a){
    float a1 = floor(abs(a)*255.);
    float b1 = fract(abs(a)*255.);
    float a2 = floor(b1*255.);
    float b2 = fract(b1*255.);
    float a3 = floor(b2*255.);
    float b3 = fract(b2*255.);
    return vec4(a1/255.,a2/255.,a3/255.,step(0., a));
    return vec4(a1/255.,a2/255.,a3/255.,b3);
}
float deCode(vec4 a){
    return (a.x+a.y/255.+a.z/(255.*255.)) * mix(-1., 1., a.w);
    return a.x+a.y/255.+a.z/(255.*255.)+a.w/(255.*255.*255.);
}

void main()
{
    // float fact = .55;
    // float sigma = .15;
    vec4 lastColor = texture2D(pyrLastTex, uv0);
    float lastValue = deCode(lastColor);
    // float lastValue = (lastColor.r);
    vec4 curColor = texture2D(pyrCurTex, uv0);
    float curValue = deCode(curColor);
    // float curValue = (curColor.r);
    vec4 resultColor = vec4(0.);// lastColor;
    float diff = lastValue - curValue;
    diff = diff + fact * diff * exp(-diff * diff / (2. * .15 * .15)) + resOffset;
    // diff = (diff + 1.) / 2.;
    diff = mix(lastValue, diff, subFlag);
    resultColor = enCode(diff);
    // resultColor = vec4(diff);
    // resultColor.a = 1.;
    gl_FragColor = resultColor;
}
