precision lowp float;
varying highp vec2 uv0;
uniform sampler2D u_albedo;
uniform sampler2D u_albedo2;
uniform float u_face_pet_type;
void main()
{
    if (u_face_pet_type == 1.0) {
        gl_FragColor = texture2D(u_albedo, uv0);
        return;
    }
    gl_FragColor = texture2D(u_albedo2, uv0);
    
    // gl_FragColor = vec4(0.2w
}
