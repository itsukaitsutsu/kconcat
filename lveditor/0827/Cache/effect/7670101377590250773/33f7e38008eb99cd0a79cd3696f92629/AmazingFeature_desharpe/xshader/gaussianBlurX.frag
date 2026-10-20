precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform float u_sampleX;
uniform float u_sigmaX;
uniform float u_stepX;

#define MAX_SAMPLE 256
const float Eps = 1e-5;

float getGaussianWeight(float x, float sigma)
{
    return exp(-0.5 * x * x / (sigma * sigma));
}

void main()
{
    // 
    // return;
    if (u_sampleX < Eps) {
        
        gl_FragColor = texture2D(u_inputTexture, uv0);
        return;
    }

    // init sumColor and sumWeight using current pixel
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float sumWeight = getGaussianWeight(0., u_sigmaX);
    vec4 sumColor = sumWeight * oriColor;
    vec2 uv = uv0;

    // loop for both direction
    for (int i = 1; i <= MAX_SAMPLE; i++) {
        float k = float(i);
        if (k > u_sampleX) {
            break;
        }

        float x = k * u_stepX;
        float weight = getGaussianWeight(x, u_sigmaX);

        // left range
        uv.x = uv0.x - x;
        if (uv.x >= 0.) {
            sumColor += weight * texture2D(u_inputTexture, uv);
            sumWeight += weight;
        }
        // right range
        uv.x = uv0.x + x;
        if (uv.x <= 1.) {
            sumColor += weight * texture2D(u_inputTexture, uv);
            sumWeight += weight;
        }
    }
    sumColor /= sumWeight;
    gl_FragColor = sumColor;
}
