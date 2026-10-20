precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;

uniform float u_vignettingIntensity;
uniform float u_vignettingParam;

const float Sqrt2 = 1.415;
const float Eps = 1e-6;

void main()
{
    vec4 color = texture2D(u_inputTexture, uv0);
    vec2 uv = 2. * uv0 - 1.;
    float dist = length(uv);

    vec2 r = vec2(0.);
    if (u_vignettingIntensity >= 0.0001) {
        r = vec2(0.1, 0.85);
    } else {
        r = vec2(0.2, 0.9);
    }

    if (u_vignettingIntensity >= 0.0001) {
        float vignettingWeight = smoothstep(mix(1., r.x, abs(u_vignettingParam)) * Sqrt2,
                                            mix(1.4, r.y, abs(u_vignettingParam)) * Sqrt2, dist);
        color.rgb = mix(color.rgb, vec3(0.), vignettingWeight);
    } else {
        float vignettingWeight = smoothstep(mix(1., r.x, abs(u_vignettingIntensity)) * Sqrt2,
                                            mix(1.5, r.y, abs(u_vignettingParam)) * Sqrt2, dist);
        color.rgb = mix(color.rgb, vec3(1.), vignettingWeight);
    }
    gl_FragColor = color;
}
