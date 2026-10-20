precision highp float;
varying highp vec2 uv0;
uniform sampler2D inputTexture;
uniform sampler2D _MainTex;
varying vec4 v_bloomPara;

uniform vec2 extraSizeForUv;

varying vec2 v_local_uv;
varying vec2 v_screen_uv;
varying vec2 m;
varying vec2 n;

uniform vec4 u_ScreenParams;

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

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

const float PI = 3.14159265359;

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
  if(borderPoint.y > 1.){
    float a = PI * 0.5 - theta;
    borderPoint = vec2(tan(a), 1.);
  }
  float borderLength = length(borderPoint);
  theta /= PI;
  float noise = gnoise((dir) * 144.44 + vec2(v_bloomPara.xy+v_bloomPara.zw*10.) * 12.2);
  noise = noise * 0.5 + 0.5;
  //   mask = step(dst, 0.9 + noise * 0.1);
//   float mask1 = smoothstep(1., noise * 0.8 + 0.2, dst);
  float mask2 = smoothstep(1.*borderLength, (noise * 0.7 + 0.3)*borderLength, dst);
  mask = mask2;
//   if (theta < 0.49) {
//     mask = 1.;
//   }
  return mask;
}

void main() {
  // vec2 uv1 = v_movingUv.xy;
  vec2 luv = getLocalUv();
  vec4 oriColor = texture2D(_MainTex, luv);
  oriColor.rgb = rgb2hsv(oriColor.rgb * 0.7);
  oriColor.g *= 0.75;
  oriColor.b *= 1.4;
  oriColor.rgb = hsv2rgb(oriColor.rgb);
  vec4 resColor = texture2D(inputTexture, uv0);
  // resColor *= getMask();
  gl_FragColor = (oriColor + resColor * (1. - oriColor.a));
//   gl_FragColor = vec4(v_bloomPara.y, 0, 0, 1);
  // vec4 res = oriColor;
  // res = vec4(luv,0,1);
  // gl_FragColor = res;

  // vec4 textCol = texture2D(_MainTex, uv0);
  // vec4 res = textCol;
  // res = vec4(uv0,0,1);

  // res = vec4(luv,0,1);
  // res = texture2D(_MainTex, luv);
  // gl_FragColor = res;
}
