precision highp float;
varying highp vec2 uv0;
varying highp vec4 v_bloomPara;

uniform sampler2D u_X2InputTex;
uniform float u_Angle;
//uniform float u_Strength;
uniform float u_BlurScale;
uniform vec4 u_ScreenParams;
uniform float blurRadius;
float normpdf(in float x, in float sigma)
{
	return 0.39894*exp(-0.5*x*x/(sigma*sigma))/sigma;
}
vec4 gaussianBlur(sampler2D i_InputTex, vec2 i_Uv, vec2 i_Dir, float i_Strength)
{
    // const int  radius = 32;
    // float s = i_Strength;
    float sigma = 4.0;
    float first = normpdf(0.0, sigma);
    float weight = 0.5 * 1.02 + 0.5;
    // weight = first;

    vec4 sum            = vec4(0.0);
    vec4 result         = vec4(0.0);
    vec2 unit_uv        = i_Dir * u_BlurScale*i_Strength;
    vec4 curColor       = texture2D(i_InputTex, i_Uv);
    vec4 centerPixel    = pow(curColor, vec4(2.2))*weight;
    // vec4 centerPixel    = curColor*weight;
    float sum_weight    = weight;

    // #ifdef SAMPLETIIMES2
    // for(int i=1;i<=SAMPLETIIMES2;i++)
    // {
    //     vec2 curRightCoordinate = i_Uv+float(i)*unit_uv;
    //     vec2 curLeftCoordinate  = i_Uv+float(-i)*unit_uv;
    //     vec4 rightColor = texture2D(i_InputTex, curRightCoordinate);
    //     vec4 leftColor = texture2D(i_InputTex, curLeftCoordinate);
    //     weight = (normpdf(float(i) / float(SAMPLETIIMES2) * 15.0, sigma) / first - 0.5) * 1.02 + 0.5;
    //     sum+=pow(rightColor, vec4(2.2))*weight;
    //     sum+=pow(leftColor, vec4(2.2))*weight;
    //     sum_weight+=weight*2.0;
    // }
    // #endif

    float sampleTime = blurRadius * i_Strength;
    for(float i=1.;i<=25.0;i+=1.0)
    {
        vec2 curRightCoordinate = i_Uv+float(i)*unit_uv;
        vec2 curLeftCoordinate  = i_Uv+float(-i)*unit_uv;
        vec4 rightColor = texture2D(i_InputTex, curRightCoordinate);
        vec4 leftColor = texture2D(i_InputTex, curLeftCoordinate);
        weight = (normpdf(float(i) / float(sampleTime) * 15.0, sigma) / first - 0.5) * 1.02 + 0.5;
        sum+=pow(rightColor, vec4(2.2))*weight;
        sum+=pow(leftColor, vec4(2.2))*weight;
        sum_weight+=weight*2.0;
        if (i>sampleTime)
        {
            break;
        }
    }

    result = (sum+centerPixel)/sum_weight; 
    // return result;
    return pow(clamp(result, 0.0, 1.0), vec4(1.0 / 2.2));
}

void main()
{
    float range = v_bloomPara.y;

    float theta = 90.0 * 3.1415926 / 180.;
    vec2 ratio = vec2(720.0) * u_ScreenParams.xy / min(u_ScreenParams.x, u_ScreenParams.y);
    vec2 dir = vec2(cos(theta), sin(theta)) / ratio;
    //vec4 color = gaussianBlur(u_X2InputTex, uv0, dir, u_Strength);
    vec4 color = gaussianBlur(u_X2InputTex, uv0, dir, range);
    gl_FragColor = color;
}
