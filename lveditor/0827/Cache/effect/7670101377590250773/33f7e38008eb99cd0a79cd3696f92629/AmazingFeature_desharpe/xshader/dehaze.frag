precision highp float;
varying vec2 uv0;

uniform sampler2D inputImageTexture;
uniform sampler2D filterBgTexture;
uniform sampler2D filterSkinTexture;
uniform sampler2D maskTexture;

uniform int imageWidth;
uniform int imageHeight;

uniform float intensity;

vec4 lm_take_effect_filter(sampler2D filterTex,vec4 inputColor,float uniAlpha)
{
  highp vec4 textureColor= inputColor;	//texture2D(inputTex,textureCoordinate);
  highp float blueColor=textureColor.b*63.;
  
  highp vec2 quad1;
  quad1.y=floor(floor(blueColor)/8.);
  quad1.x=floor(blueColor)-(quad1.y*8.);
  
  highp vec2 quad2;
  quad2.y=floor(ceil(blueColor)/8.);
  quad2.x=ceil(blueColor)-(quad2.y*8.);
  
  highp vec2 texPos1;
  texPos1.x=(quad1.x*1./8.)+.5/512.+((1./8.-1./512.)*textureColor.r);
  texPos1.y=(quad1.y*1./8.)+.5/512.+((1./8.-1./512.)*textureColor.g);
  
  highp vec2 texPos2;
  texPos2.x=(quad2.x*1./8.)+.5/512.+((1./8.-1./512.)*textureColor.r);
  texPos2.y=(quad2.y*1./8.)+.5/512.+((1./8.-1./512.)*textureColor.g);
  
  vec4 newColor1=texture2D(filterTex,texPos1);
  vec4 newColor2=texture2D(filterTex,texPos2);
  vec4 newColor=mix(newColor1,newColor2,fract(blueColor));
  newColor = mix(textureColor,vec4(newColor.rgb,textureColor.w),uniAlpha);

  return newColor;
}

void main(void) 
{
    vec2 maskCoord = vec2(uv0.x, 1.0 - uv0.y);
    float mask = texture2D(maskTexture, maskCoord).a;
    // vec2 screenSize = vec2(imageWidth, imageHeight);
    vec2 uv = uv0;
    vec4 color = texture2D(inputImageTexture, uv);
    // vec4 filter =  texture2D(inputImageTexture2, uv);
    vec4 bgResultColor = lm_take_effect_filter(filterBgTexture, color, intensity);
    vec4 skinResultColor = lm_take_effect_filter(filterSkinTexture, color, intensity);
    vec4 resultColor = mix(bgResultColor, skinResultColor, mask);
    gl_FragColor = resultColor;
    gl_FragColor = clamp(gl_FragColor, 0.0, gl_FragColor.a);

    // gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
