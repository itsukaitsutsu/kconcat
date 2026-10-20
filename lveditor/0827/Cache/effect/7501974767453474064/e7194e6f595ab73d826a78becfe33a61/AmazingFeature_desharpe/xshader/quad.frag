precision highp float;
varying vec2 uv0;

uniform sampler2D inputImageTexture;
uniform sampler2D filterMaskTexture;
uniform sampler2D maskTexture;
uniform sampler2D grayTexture;

uniform int imageWidth;
uniform int imageHeight;

uniform float intensity;

vec3 adjust_contrast(vec3 rgb, float p) {
    vec3 y = pow(rgb, vec3(p)) / (pow(rgb, vec3(p)) + pow(1.0 - rgb, vec3(p)));
    return y;
}

vec3 soft_light_blend(vec3 base, vec3 blend) 
{
    vec3 result = vec3(0.0);
    for (int i = 0; i < 3; i++) {
        if (blend[i] < 0.5) {
            result[i] = 2.0 * base[i] * blend[i] + base[i] * base[i] * (1.0 - 2.0 * blend[i]);
        } else {
            result[i] = 2.0 * base[i] * (1.0 - blend[i]) + sqrt(base[i]) * (2.0 * blend[i] - 1.0);
        }
    }
    return result;
}
void main()
{
    float mask = texture2D(maskTexture, uv0).r;
    if (mask > 0.5) {
        gl_FragColor = texture2D(inputImageTexture, uv0);
        return;
    }
    vec4 grayColor = texture2D(grayTexture, uv0);
    vec4 bluredColor = texture2D(filterMaskTexture, uv0);
    vec4 hightPassColor = clamp( grayColor - bluredColor + vec4(0.5),vec4(0.0),vec4(1.0));
    hightPassColor.rgb = adjust_contrast(hightPassColor.rgb, 1.0+intensity*0.5);
    vec4 inputColor = texture2D(inputImageTexture, uv0);
    

    vec3 adjustColor = soft_light_blend(inputColor.rgb, hightPassColor.rgb);
    gl_FragColor = mix(inputColor, vec4(adjustColor,inputColor.a), intensity* (1.0 - mask));
    // gl_FragColor = mix(inputColor, adjustColor, mask);
    // gl_FragColor = mix(texture2D(maskTexture, uv0), gl_FragColor, mask);
}