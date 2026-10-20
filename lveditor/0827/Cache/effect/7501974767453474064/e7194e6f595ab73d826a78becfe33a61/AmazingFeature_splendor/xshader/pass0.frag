precision highp float;

uniform sampler2D inputImageTexture;
varying vec2 uv;

uniform int inputWidth;
uniform int inputHeight;

uniform float iTime;
uniform sampler2D nLut1;
uniform sampler2D nLut2;
uniform sampler2D pLut1;
uniform sampler2D pLut2;
uniform float ins;

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

float v_LUT(float x,sampler2D Lut,float param){
    return texture2D(Lut,vec2(x,param)).r;
}
vec3 rgb_Lut(vec3 col,sampler2D Lut,float param){
    return vec3(texture2D(Lut,vec2(col.r,param)).r,
                texture2D(Lut,vec2(col.g,param)).r,
                texture2D(Lut,vec2(col.b,param)).r
                                                 );
}
float get_shadow_map(float image_v, float shadow_percent){
    float shadow_tone = shadow_percent ;
    float shadow_map = 1.0 - image_v / shadow_percent;
    if(image_v >= shadow_tone)
        shadow_map = 0.0;
    return shadow_map;
}
vec3 adjust_positive(vec3 colimg,float param){
    // param = max(param, 1)
    // lut1 = lut_map_1[param - 1]
    // lut2 = lut_map_2[param - 1]

    // # 1st curve, adjust with shadow_map
    vec3 image_hsv = rgb2hsv(colimg);
    float shadow_map = get_shadow_map(image_hsv.z, 0.8);
    // shadow_map = np.expand_dims(shadow_map, axis=2)
    image_hsv.z = v_LUT(image_hsv.z, pLut1,param);
    vec3 image_lut = hsv2rgb(image_hsv);
    //vec3 image = (1. - shadow_map) * colimg + shadow_map * image_lut;
    vec3 image = mix(colimg,image_lut,shadow_map);
    // image = float_to_uint8(image, scale=1.0)

    //  2nd curve
    image = rgb_Lut(image, pLut2,param);
    return image;
}

vec3  adjust_negative(vec3 colimg,float param){
    // param = max(param, 1)
    // lut1 = lut_map_1[param - 1]
    // lut2 = lut_map_2[param - 1]

    // # 1st curve, adjust with shadow_map
    vec3 image_hsv = rgb2hsv(colimg);
    float shadow_map = get_shadow_map(image_hsv.z, 0.9);
    float image_v_lut = v_LUT(image_hsv.z, nLut1,param);
    // image_v_lut = (1 - shadow_map) * image_hsv[:, :, 2] + 
    //               shadow_map * image_v_lut
    image_v_lut=mix(image_hsv.z,image_v_lut,shadow_map);
    //image_v_lut = float_to_uint8(image_v_lut, scale=1.0)
    image_hsv.z = image_v_lut;
    vec3 image = hsv2rgb(image_hsv);

    // # 2nd curve
    image = rgb_Lut(image, nLut2,param);
    return image;
}
void main()
{
    vec4 col = texture2D(inputImageTexture,uv);
    float myins = (ins-0.5)*2.0;
    float param = abs(myins);
    vec3 pcol = adjust_positive(col.rgb,param);
    vec3 ncol = adjust_negative(col.rgb,param);
    col.rgb = mix(ncol,pcol,step(0.0,myins));
    gl_FragColor = vec4(col);
}
