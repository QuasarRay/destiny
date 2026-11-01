#ifndef DESTINY_QUATERNION
#define DESTINY_QUATERNION

typedef struct QuaternionD
{
public:
    QuaternionD() : w(1.0) {};
    QuaternionD( double _w, double _x, double _y, double _z ) {w = _w; v.x = _x; v.y = _y; v.z = _z;};
	QuaternionD(  Vector3d axis, double angle) { 
		if(axis.LengthSq()!=1.0)
			axis.Normalize();
		w = cos(angle*0.5);
		v = sin(angle*0.5)*axis; };
	QuaternionD(  double _w, const Vector3d& _v) { w = _w; v = _v; };

    // assignment operators
    QuaternionD& operator += ( const QuaternionD& q) { w+=q.w; v+=q.v; return *this;};
    QuaternionD& operator -= ( const QuaternionD& q) { w-=q.w; v-=q.v; return *this;};

    // inverse operator
    QuaternionD operator - () const { 
				return QuaternionD(w,-v)/LengthSq();
			};

	QuaternionD operator / (double c) const {
				c = 1.0/c;
				return QuaternionD(w*c, v*c);
				};

    // multiplication operators
	QuaternionD operator * (const QuaternionD& q) const {
		return QuaternionD(
             w * q.w - v.x * q.v.x - v.y * q.v.y - v.z * q.v.z,
             w * q.v.x + v.x * q.w + v.y * q.v.z - v.z * q.v.y,
             w * q.v.y + v.y * q.w + v.z * q.v.x - v.x * q.v.z,
             w * q.v.z + v.z * q.w + v.x * q.v.y - v.y * q.v.x
			 );
	};

    QuaternionD operator + ( const QuaternionD& q) const { return QuaternionD(w+q.w, v+q.v);};
    QuaternionD operator - ( const QuaternionD& q) const { return QuaternionD(w-q.w, v-q.v);};

    bool operator == ( const QuaternionD& q) const  { return w==q.w && v==q.v;};
    bool operator != ( const QuaternionD& q) const {return w!=q.w || v!=q.v;};

	double LengthSq() const { return w*w + v.LengthSq();};

	Vector3d RotateVector(const Vector3d &v) const {
					QuaternionD q = (*this)*QuaternionD(0.0,v)*(-*this);
					return q.v;
					};

	double w;
	Vector3d v;
} QuaternionD;

#endif