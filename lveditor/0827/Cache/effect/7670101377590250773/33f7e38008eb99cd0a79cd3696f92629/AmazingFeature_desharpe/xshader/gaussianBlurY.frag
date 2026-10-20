precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform float u_sampleY;
uniform float u_sigmaY;
uniform float u_stepY;

#define MAX_SAMPLE 256
const float Eps = 1e-5;

float getGaussianWeight(float x, float sigma)
{
    return exp(-0.5 * x * x / (sigma * sigma));
}

void main()
{
    if (u_sampleY < Eps) {
        gl_FragColor = texture2D(u_inputTexture, uv0);
        return;
    }

    // init sumColor and sumWeight using current pixel
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float sumWeight = getGaussianWeight(0., u_sigmaY);
    vec4 sumColor = sumWeight * oriColor;
    vec2 uv = uv0;

    // loop for both direction
    for (int i = 1; i <= MAX_SAMPLE; i++) {
        float k = float(i);
        if (k > u_sampleY) {
            break;
        }

        float y = k * u_stepY;
        float weight = getGaussianWeight(y, u_sigmaY);

        // left range
        uv.y = uv0.y - y;
        if (uv.y >= 0.) {
            sumColor += weight * texture2D(u_inputTexture, uv);
            sumWeight += weight;
        }
        // right range
        uv.y = uv0.y + y;
        if (uv.y <= 1.) {
            sumColor += weight * texture2D(u_inputTexture, uv);
            sumWeight += weight;
        }
    }
    sumColor /= sumWeight;
    gl_FragColor = sumColor;
    // gl_FragColor = vec4(1.0);
}
