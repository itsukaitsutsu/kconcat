precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform float u_airLightThreshold;
uniform vec4 u_ScreenParams;

#define ITER 200
const float Eps = 1e-5;

void main()
{
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float airLight = 0.;
    float count = 0.;

    float dx = 1. / float(ITER);
    float dy = dx;

    // output texture size is 1x1
    for (int i = 0; i < ITER; i++) {
        float y = float(2 * i + 1) * 0.5 * dy;
        for (int j = 0; j < ITER; j++) {
            float x = float(2 * j + 1) * 0.5 * dx;
            float color = texture2D(u_inputTexture, vec2(x, y)).r;
            if (color > u_airLightThreshold) {
                airLight += color;
                count += 1.;
            }
        }
    }
    airLight = airLight / max(count, Eps);
    airLight = max(airLight, u_airLightThreshold);

    oriColor.rgb = vec3(airLight);
    gl_FragColor = oriColor;
}
