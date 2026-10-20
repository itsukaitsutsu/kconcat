precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;

uniform float u_threshold;
uniform float u_stepX;
uniform float u_stepY;

#define RADIUS 3
const float Eps = 1e-5;

vec4 normalizeColor(vec4 sumColor, vec4 sumWeight, vec4 centerColor)
{
    sumColor.r = sumWeight.r < Eps ? centerColor.r : sumColor.r / sumWeight.r;
    sumColor.g = sumWeight.g < Eps ? centerColor.g : sumColor.g / sumWeight.g;
    sumColor.b = sumWeight.b < Eps ? centerColor.b : sumColor.b / sumWeight.b;
    sumColor.a = sumWeight.a < Eps ? centerColor.a : sumColor.a / sumWeight.a;
    return sumColor;
}

void main()
{
    vec4 sumColor = vec4(0.);
    vec4 sumWeight = vec4(0.);
    vec4 centerColor = texture2D(u_inputTexture, uv0);
    float threshold = max(u_threshold, Eps);
    for (int row = -RADIUS; row <= RADIUS; row++) {
        float offsetY = float(row) * u_stepY;
        for (int col = -RADIUS; col <= RADIUS; col++) {
            float offsetX = float(col) * u_stepX;
            vec2 uv = uv0 + vec2(offsetX, offsetY);
            vec4 color = texture2D(u_inputTexture, uv);
            vec4 weight = 1. - abs(color - centerColor) / (2.5 * threshold);
            weight = max(weight, vec4(0.));
            sumColor += color * weight;
            sumWeight += weight;
        }
    }
    sumColor = normalizeColor(sumColor, sumWeight, centerColor);

    gl_FragColor = clamp(sumColor, 0., 1.);
}
