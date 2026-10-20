

 precision highp float;
 varying vec2 texCoord;
 varying vec2 sucaiTexCoord;
 uniform float opacity;


//  uniform sampler2D sucaiImageTexture;
uniform sampler2D maskImg1;
uniform sampler2D maskImg2;
uniform sampler2D maskImg3;
uniform sampler2D maskImg4;

 uniform float intensity;

 



 void main(void)
 {

    //  vec4 sucai = texture2D(sucaiImageTexture, sucaiTexCoord);
    float mask1 = texture2D(maskImg1, sucaiTexCoord).r;
    float mask2 = texture2D(maskImg2, sucaiTexCoord).r;
    float mask3 = texture2D(maskImg3, sucaiTexCoord).r;


     gl_FragColor = vec4(mask1,mask2,mask3,max(max(mask1,mask2),mask3));
    // gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
 }
 