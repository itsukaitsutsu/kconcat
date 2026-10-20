precision highp float;
varying highp vec2 uv0;
uniform sampler2D upTex;
uniform sampler2D preTex;
uniform float nlevel;
uniform float addFlag;

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z +  (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

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
    vec4 upColor = texture2D(upTex, uv0);
    vec4 preColor = texture2D(preTex, uv0);
    // float upValue = upColor.r;
    float upValue = deCode(upColor);
    // float preValue = preColor.r;
    float preValue = deCode(preColor);
    float resultValue = upValue * addFlag + mix(preValue, preValue, addFlag);

    // vec4 resultColor = vec4(resultValue);
    // resultColor.a = 1.;
    vec4 resultColor = enCode(clamp(resultValue, 0., 1.));
    gl_FragColor = resultColor;
}
