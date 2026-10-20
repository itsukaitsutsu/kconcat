precision highp float;
varying highp vec2 uv0;
varying highp vec4 v_color;
varying highp vec4 v_bloomPara;
varying highp vec4 v_bloomPara2;
uniform sampler2D _MainTex;
uniform float u_Strength;
uniform float sizeScale;
uniform vec2 u_Center;
uniform float u_light;
uniform float test;
uniform float blurscale;
uniform vec4 u_ScreenParams;

varying vec2 v_local_uv;
varying vec2 v_screen_uv;
varying vec2 m;
varying vec2 n;

vec2 getLocalUv() {
  vec2 uv = v_local_uv;
  vec2 x = vec2(0.0);
  vec2 y = vec2(0.0);
  x = (m + n) / (2.0 * (v_screen_uv));
  y = (m - n) / (2.0 * (1. - v_screen_uv));
  float adapt_width = x.x - y.x;
  float adapt_height = x.y - y.y;
  uv.x -= (x.x + y.x) * 0.5;
  uv.y += (x.y + y.y) * 0.5;
  uv.x /= (adapt_width * 0.5);
  uv.y /= (adapt_height * 0.5);
  uv = uv * 0.5 + 0.5;
  // uv.x = 1.-uv.x;
  uv.y = 1. - uv.y;
  return uv;
}

float hash(vec2 p) // replace this by something better
{
  p = 50.0 * fract(p * 0.3183099 + vec2(0.71, 0.113));
  return -1.0 + 2.0 * fract(p.x * p.y * (p.x + p.y));
}

// gradient noise
float gnoise(in vec2 p) {

  vec2 i = floor(p);
  vec2 f = fract(p);

  // quintic interpolant
  //   vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  // cubic interpolant
  vec2 u = f * f * (3.0 - 2.0 * f);

  return mix(mix(hash(i + vec2(0.0, 0.0)), hash(i + vec2(1.0, 0.0)), u.x),
             mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

const float PI = 3.14159265;

float getMask() {
  vec2 uv = getLocalUv();
  uv -= 0.5;
  uv *= 2.;
  uv = abs(uv);
  float space = 0.1;
  float mask =
      smoothstep(1., 1. - space, uv.x) *
      smoothstep(1., 1. - space * u_ScreenParams.x / u_ScreenParams.y, uv.y);
  //   mask = 1.;

  float dst = length(uv);

  float totalLength = pow(2., 0.5);
  float theta = atan(uv.y, uv.x);
  vec2 dir = vec2(cos(theta), sin(theta));
  vec2 borderPoint = vec2(1., tan(theta));
  if (borderPoint.y > 1.) {
    float a = PI * 0.5 - theta;
    borderPoint = vec2(tan(a), 1.);
  }
  float borderLength = length(borderPoint);
  theta /= PI;
  float noise = gnoise((dir) * 144.44 +
                       vec2(v_bloomPara.xy + v_bloomPara.zw * 10.) * 12.2);
  noise = noise * 0.5 + 0.5;
  noise *= 0.7;
  //   mask = step(dst, 0.9 + noise * 0.1);
  //   float mask1 = smoothstep(1., noise * 0.8 + 0.2, dst);
  float mask2 =
      smoothstep(1.25 * borderLength, (noise * 0.5 + 0.5) * borderLength, dst);
  mask = mask2;
  mask = 1.;
  return mask;
}

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 ChangeHue(vec3 col, float h) {
  float shift = 2.0 * h * 3.1415926535;
  vec3 m = vec3(cos(shift), -sin(shift) * .57735, 0);
  m = vec3(m.xy, -m.y) + (1. - m.x) * .33333;
  return mat3(m, m.zxy, m.yzx) * col;
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 radialBlur(sampler2D i_InputTexture, vec2 uv, float i_Strength,
                vec2 i_Center, float i_light, float i_enableColorCustomized) {
  float decay = 0.9;
  float density = i_Strength;
  float weight = 0.58767;
  const int nsamples = 64;

  vec2 tc = uv.xy;
  vec2 deltaTexCoord = tc - i_Center.xy;

  float blurstep = blurscale;
  if (blurstep < 1.0) {
    blurstep = 1.0;
  }

  float mask = getMask();
  deltaTexCoord *= (1.0 / float(nsamples) * density) * 1.60 * mask;
  float illuminationDecay = 1.0;
  float sumweight = 0.0;
  vec4 color = vec4(0.0);
  float lightStrength = 0.1;
  float noise = fract(sin(dot(uv.xy, vec2(12.9898, 78.233))) * 43758.5453);
  // noise = 1.;
  tc += deltaTexCoord * noise;

  //   tc = clamp(tc, 0., 1.);
  for (int i = 0; i < 64; i++) {
    tc -= deltaTexCoord;
    vec4 sampl = texture2D(i_InputTexture, tc.xy) * lightStrength;
    sampl *= illuminationDecay * weight;
    color += sampl;
    sumweight += illuminationDecay * weight * lightStrength;
    illuminationDecay *= decay;
  }
  vec4 oriColor = texture2D(i_InputTexture, uv.xy) * 0.7;
  oriColor.rgb = rgb2hsv(oriColor.rgb);
  oriColor.g *= 0.75;
  oriColor.b *= 1.4;
  oriColor.rgb = hsv2rgb(oriColor.rgb);
  if (i_enableColorCustomized > 0.5) {
    color.rgb = v_color.rgb * color.a * v_color.a;
  }
  color = clamp(i_light * pow(color, vec4(1.0 / (0.01 + i_light))), 0.0, 1.0);
  return oriColor + color / sumweight * 1.0 *
                        (1. - clamp(oriColor.a, 0.0, 1.0)) * i_Strength;
}

void main() {
  vec2 luv = getLocalUv();

  float range = pow(v_bloomPara.y, 0.2);
  float light = pow(v_bloomPara.x, 0.2) * 1.5;
  float dirX = 1.0 - v_bloomPara.z;
  float dirY = 1.0 - v_bloomPara.w;
  float enableColorCustomized = v_bloomPara2.x;
  // gl_FragColor = vec4(1.0);
  // gl_FragColor = radialBlur(_MainTex, uv0, u_Strength * 0.75, u_Center);
  gl_FragColor = radialBlur(_MainTex, luv, range, vec2(dirX, dirY), light,
                            enableColorCustomized);
  // gl_FragColor = vec4(test);
}
