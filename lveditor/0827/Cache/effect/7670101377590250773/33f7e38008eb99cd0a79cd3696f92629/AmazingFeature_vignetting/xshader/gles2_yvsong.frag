precision highp float;

varying vec2 fTexCoord;
uniform sampler2D _MainTex;
void main() 
{
    gl_FragColor = texture2D(_MainTex, fTexCoord);
}
