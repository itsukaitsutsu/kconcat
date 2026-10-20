precision highp float;
varying highp vec2 uv0;

#define MAX_POINTS 25*3
uniform vec2 controlPoints[MAX_POINTS];
uniform int pointsNums;


vec2 getControlPoints(int index)
{
    vec2 res = vec2(0.0);
    for (int i = 0; i < 75; i++)
    {
        if (index == i)
        {
            res = controlPoints[i];
            break;
        }
    }
    return res;
}

float vec2ToFloat(vec2 val){
    float res = val.x + val.y/255.0;
    return res;
}
vec2 floatToVec2(float val){
    float a = floor(val*255.0)/255.0;
    float b = fract(val*255.0);
    return vec2(a, b);
}

float vec4ToFloat(vec4 val){
    float res = val.x * 255.0 + val.y + val.z/255.0;
    res = val.z < 0.5 ? res : -res;
    return res;
}
vec4 floatToVec4(float val){
    float d = val < 0.?1.:0.;
    float a = floor(abs(val))/255.0;
    float b = floor(fract(abs(val))*255.0)/255.0;
    float c = fract(fract(abs(val))*255.0);
    return vec4(a, b, c, d);
}

float Linear(float x1, float y1, float x2, float y2, float x)
{
    if (x2 <= x1)
    {
        if (x > x2)
        {
            return y2;
        }
        else
        {
            return y1;
        }
    }
    return (x-x1)*(y2-y1)/(x2-x1) + y1;
}

float Bezier(float p0, float p1, float p2, float p3, float t)
{
    float result;
    float p0p1 = (1.0 - t) * p0 + t * p1;
    float p1p2 = (1.0 - t) * p1 + t * p2;
    float p2p3 = (1.0 - t) * p2 + t * p3;
    float p0p1p2 = (1.0 - t) * p0p1 + t * p1p2;
    float p1p2p3 = (1.0 - t) * p1p2 + t * p2p3;
    result = (1.0 - t) * p0p1p2 + t * p1p2p3;
    return result;
}

const float ESP = 0.00005;
float computeT2(float p0, float p1, float p2, float p3, float p) 
{
    if (abs(p0 - p) < ESP)
    {
        return 0.0;
    }    
    if (abs(p3 - p) < ESP)
    {
        return 1.0;
    }
    float startT = 0.0;
    float endT = 1.0;
    float halfT = 0.5;

    float halfP = Bezier(p0, p1, p2, p3, halfT);

    for (int i =0; i < 20;i++)
    {
        if (abs(halfP - p) < ESP)
        {
            break;
        }
        if(halfP < p){
            startT = halfT;
        }else {
            endT = halfT;
        }
        halfT = (startT + endT)/2.0;
        halfP = Bezier(p0, p1, p2, p3, halfT);
    }
    return halfT;
}

float genBezierY(float x)
{
    int nums = 0;
    for (int j = 0; j < 75; j++)
    {
        nums = j;
        if (j >= pointsNums)
        {
            break;
        }
    }
    if(x < getControlPoints(0).x)
    {
        return Linear(getControlPoints(0).x, getControlPoints(0).y, getControlPoints(1).x, getControlPoints(1).y, x);
    }
    else if(x > getControlPoints((nums-1) * 3).x)
    {
        return Linear(getControlPoints((nums-1) * 3 - 1).x, getControlPoints((nums-1) * 3 - 1).y, getControlPoints((nums-1) * 3).x, getControlPoints((nums-1) * 3).y, x);
    }

    int i = 0;
    for (int j = 0; j < 75; j++)
    {
        i = j;
        if (j >= pointsNums)
        {
            break;
        }
        if (x < getControlPoints(j*3).x)
        {
            break;
        }
    }
    vec2 p0 = getControlPoints((i-1)*3);
    vec2 p1 = getControlPoints((i-1)*3 + 1);
    vec2 p2 = getControlPoints((i-1)*3 + 2);
    vec2 p3 = getControlPoints(i*3);
    float t = computeT2(p0.x, p1.x, p2.x, p3.x, x);
    float y = Bezier(p0.y, p1.y, p2.y, p3.y, t);
    return y;
}

void main(void) {
    float y = genBezierY(uv0.x);
    gl_FragColor = floatToVec4(y);
    // gl_FragColor = vec4(floatToVec2(y), 1.0, 1.0);
    // gl_FragColor = vec4(0, uv0.x, 0, 0.0);
}
