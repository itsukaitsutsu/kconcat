precision highp float;
varying highp vec2 uv0;
uniform sampler2D inputTextureY;
uniform vec4 u_ScreenParams;
uniform float u_StrengthY;
uniform float blurscale1;
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
vec4 gauss_blur(sampler2D inputTexture, vec2 uv, float angle, float blurSize, vec2 uRenderSize)
{
    float half_gaussian_weight[9];

    float radian = 3.1415926 * angle / 180.0;
    vec2 dir = vec2(cos(radian), sin(radian));
    
    half_gaussian_weight[0]= 0.2;   //0.2;//0.137401;
    half_gaussian_weight[1]= 0.19;  //0.2;//0.125794;
    half_gaussian_weight[2]= 0.17;  //0.2;//0.106483;
    half_gaussian_weight[3]= 0.15;  //0.2;//0.080657;
    half_gaussian_weight[4]= 0.13;  //0.2;//0.054670;
    half_gaussian_weight[5]= 0.11;  //0.2;//0.033159;
    half_gaussian_weight[6]= 0.08;  //0.2;//0.017997;
    half_gaussian_weight[7]= 0.05;  //0.2;//0.008741;
    half_gaussian_weight[8]= 0.02;  //0.2;//0.003799;

    vec4 sum            = vec4(0.0);
    vec4 result         = vec4(0.0);
    vec2 unit_uv        = vec2(blurSize / uRenderSize.x, blurSize / uRenderSize.y)*1.25;
    vec4 centerPixel    = texture2D(inputTexture, uv) * half_gaussian_weight[0];
    float sum_weight    = half_gaussian_weight[0];

    vec2 curPositiveCoordinate = uv;
    vec2 curNegativeCoordinate = uv;

    for(int i=1; i<9; i++)
    {
        curPositiveCoordinate    += dir * unit_uv;
        curNegativeCoordinate    -= dir * unit_uv;
        sum += texture2D(inputTexture, curPositiveCoordinate) * half_gaussian_weight[i];
        sum += texture2D(inputTexture, curNegativeCoordinate) * half_gaussian_weight[i];
        sum_weight += half_gaussian_weight[i] * 2.0;
    }
    result = (sum + centerPixel) / sum_weight;
    return result;
}
void main()
{
    vec4 resColor = gauss_blur(inputTextureY, uv0, 90.0, u_StrengthY*blurscale1, u_ScreenParams.xy);
    // resColor.rgb = pow(resColor.rgb,vec3(0.));
    gl_FragColor = resColor*1.1; 
}
