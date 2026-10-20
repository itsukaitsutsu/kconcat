precision highp float;
varying highp vec2 uv0;
uniform sampler2D inputImageTexture;
uniform float sharpness;
uniform vec4 u_ScreenParams;
const int M = 1;
uniform float white_gam;
uniform float black_gam;

const mat3 rgb2ycbcr = mat3(
    0.299, -0.168736, 0.5,
    0.587, -0.331264, -0.418688,
    0.114, 0.5, -0.081312
);

const mat3 ycbcr2rgb = mat3(
    1.0, 1.0, 1.0,
    0.0, -0.344136, 1.772,
    1.402, -0.714136, 0.0
);

void main()
{
    vec2 step = vec2(1.0 / u_ScreenParams.x, 1.0 / u_ScreenParams.y);
    vec4 srcColor = texture2D(inputImageTexture, uv0);
    highp vec3 color = vec3(0);
    // float coef = (sum - 1.0) / ((2.0 * M + 1.0) * (2.0 * M + 1.0));

    vec3 p1 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(-1, -1)).rgb);
    vec3 p2 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(0, -1)).rgb);
    vec3 p3 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(1, -1)).rgb);
    vec3 p4 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(-1, 0)).rgb);
    vec3 p5 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(0, 0)).rgb);
    vec3 p6 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(1, 0)).rgb);
    vec3 p7 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(-1, 1)).rgb);
    vec3 p8 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(0, 1)).rgb);
    vec3 p9 = rgb2ycbcr * (texture2D(inputImageTexture, uv0 + step * vec2(1, 1)).rgb);

    float f1 = 2.0 * p5.x - p4.x - p6.x;
    float f2 = 2.0 * p5.x - p2.x - p8.x;
    float f3 = 2.0 * p5.x - p3.x - p7.x;
    float f4 = 2.0 * p5.x - p1.x - p9.x;

    float maxf = max(max(max(abs(f1), abs(f2)), abs(f3)), abs(f4));

    color = ycbcr2rgb * (p5);

    if (abs(abs(f1) - maxf) <= 0.01)
    {
        if (f1 > 0.0)
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f1 * white_gam, 0, 0)));
        else
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f1 * black_gam, 0, 0)));
    }
    else if (abs(abs(f2) - maxf) <= 0.01)
    {
        if (f2 > 0.0)
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f2 * white_gam, 0, 0)));
        else
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f2 * black_gam, 0, 0)));
    }
    else if (abs(abs(f3) - maxf) <= 0.01)
    {
        if (f3 > 0.0)
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f3 * white_gam, 0, 0)));
        else
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f3 * black_gam, 0, 0)));
    }
    else if (abs(abs(f4) - maxf) <= 0.01)
    {
        if (f4 > 0.0)
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f4 * white_gam, 0, 0)));
        else
            color = ycbcr2rgb * ((p5 + vec3(sharpness * f4 * black_gam, 0, 0)));
    }
    // gl_FragColor = vec4(color, 1.0);
    gl_FragColor = vec4(clamp(color, 0., 1.), srcColor.a);
    // gl_FragColor = abs(gl_FragColor - srcColor);
}
