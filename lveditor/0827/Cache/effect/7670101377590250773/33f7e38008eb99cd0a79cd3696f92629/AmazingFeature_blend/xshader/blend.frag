precision lowp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTex;
uniform sampler2D u_resultTex;
uniform sampler2D u_adjustTex;
uniform sampler2D u_previewMask;
uniform int u_blendWithMask;
uniform vec4 u_maskPreviewColor;
uniform float u_maskBlendMode;
uniform vec2 u_position;
uniform vec2 u_scale;
uniform float u_rotation;
uniform float u_aspect;

vec2 rotate(vec2 coord, float angle) {
    return vec2(cos(angle) * coord.x - sin(angle) * coord.y,
                sin(angle) * coord.x + cos(angle) * coord.y);
}

vec2 transformUV(vec2 uv) {
    uv = vec2((uv.x - 0.5) * 2.0 * u_aspect, (uv.y - 0.5) * 2.0);
    uv = vec2(uv.x - u_position.x * u_aspect, uv.y + u_position.y);
    uv = rotate(uv, u_rotation);
    uv = vec2(uv.x / u_scale.x, uv.y / u_scale.y);
    uv = vec2(uv.x * 0.5 / u_aspect + 0.5, uv.y * 0.5 + 0.5);
    return uv;
}


void main()
{
    if (u_blendWithMask == 0) {
        gl_FragColor = texture2D(u_resultTex, uv0);
    }
    else if (u_maskBlendMode < 0.001){
        vec4 inputColor = texture2D(u_inputTex, uv0);
        vec4 resultColor = texture2D(u_resultTex, uv0);
        vec4 adjustColor = texture2D(u_adjustTex, uv0);
        vec4 fColor = vec4(mix(adjustColor.rgb,u_maskPreviewColor.rgb,u_maskPreviewColor.a), adjustColor.a);
        resultColor.rgb = mix(fColor.rgb, inputColor.rgb, resultColor.a);
        resultColor.a = inputColor.a;
        gl_FragColor = resultColor;
    }
    else
    {
        vec4 inputColor = texture2D(u_inputTex, uv0);
        vec4 resultColor = texture2D(u_resultTex, uv0);
        vec4 adjustColor = texture2D(u_adjustTex, uv0);
        vec4 previewMask = vec4(0.0);
        vec2 transformedUV = transformUV(uv0);
        if (transformedUV.x < 1.0 && transformedUV.y < 1.0 && transformedUV.x > 0.0 && transformedUV.y > 0.0)
        {
            previewMask = texture2D(u_previewMask, transformedUV);
        }
        vec4 selectColor = vec4(mix(adjustColor.rgb, inputColor.rgb, previewMask.a), adjustColor.a);
        vec4 selectPreviewColor = vec4(mix(vec3(0.0, 0.0, 0.0), u_maskPreviewColor.rgb, previewMask.a), u_maskPreviewColor.a);
        vec4 fColor = vec4(mix(selectColor.rgb,selectPreviewColor.rgb,selectPreviewColor.a * previewMask.a), selectColor.a);
        resultColor.rgb = mix(fColor.rgb, inputColor.rgb, resultColor.a);
        resultColor.a = inputColor.a;
        gl_FragColor = resultColor;
    }
}
