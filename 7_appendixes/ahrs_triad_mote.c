
/*
 *
 *  Copyright (c) 2013, Giuseppe Torre
 *  All rights reserved.
 *
 *  Redistribution and use in source and binary forms, with or without
 *  modification, are permitted provided that the following conditions are met:
 *      * Redistributions of source code must retain the above copyright
 *        notice, this list of conditions and the following disclaimer.
 *      * Redistributions in binary form must reproduce the above copyright
 *        notice, this list of conditions and the following disclaimer in the
 *        documentation and/or other materials provided with the distribution.
 *      * Neither the name of the BREAKFAST LLC nor the
 *        names of its contributors may be used to endorse or promote products
 *        derived from this software without specific prior written permission.
 *
 *  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
 *  ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 *  WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 *  DISCLAIMED. IN NO EVENT SHALL BREAKFAST LLC BE LIABLE FOR ANY
 *  DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 *  (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 *  LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 *  ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 *  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 *  SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */ 



#include "ext.h"    
#include "ext_mess.h"
#include <string.h>
#include <stdio.h>
#include <math.h>
#define EPSILON 1e-6

//******************CALIBRATION STUFF*************************//
/* INTRODUCE THE OFFSET VALUES IN THE FOLLOWING VARIABLES IN ADC values */
#define ACCELX_OFFSET 2043 //initial offset in accelerometer X
#define ACCELY_OFFSET 2053 // initial offset in accelerometer Y
#define ACCELZ_OFFSET 2264//initial offset in accelerometer Z

#define MagX_OFFSET 1917 //initial offset in Magnetometer X
#define MagY_OFFSET 1813 // initial offset in Magnetometer Y
#define MagZ_OFFSET 1667//initial offset in Magnetometer Z
/* WE SHOULD CALIBRATE THE FOLLOWING VALUES FOR EACH PARTICULAR IMU axis */
#define ACCELX_Resolution 536 
#define ACCELY_Resolution 661 
#define ACCELZ_Resolution 559
#define MagX_Resolution 186
#define MagY_Resolution 189
#define MagZ_Resolution 170
//****************** END CALIBRATION STUFF  *************************//


void *this_class; // Required. Global pointing to this class  

typedef struct _triad // Data structure for this object  
{ 
	t_object m_ob;	 // Must always be the first field; used by Max  
	
	Atom m_args[9];	 // we want our inlet to be receiving a list of 10 elements
	
	long m_value;	 //inlet
					
	void *m_R1;		 //these are all the outlets for the 3 X 3 Matrix
	void *m_R2;		 // R1 --> firts top left cell ..R2 middle cell of the first row ...and so on
	void *m_R3;					//
	void *m_R4;					// End Outlets -----------------------------------------	
} t_triad; 
			
			void *triad_new(long value);																	
			void triad_assist(t_triad *triad, void *b, long msg, long arg, char *s);			
			void triad_free(t_triad *triad);													
			
			void triad_list(t_triad *x, Symbol *s, short argc, t_atom *argv);					


																												
			void MatrixByMatrix(double *Result,double *MatrixLeft,double *MatrixRight);							
			void Matrix2Quat(double *Quat,double *Matrix);
			void Quat2Matrix(double *Matrix, double *Quat); 
			void inverseQuat(double *InvQuat, double *RegQuat); 
			void NormQuat(double *YesQuat, double *NotQuat);
			void Slerp(double *NewQuat, double *OldQuat, double *CurrentQuat);
			void NormVect(double *YesVect, double *NotVect);
			double orientationMatrix[9];
			double Result[9];
			int    i;
			double InvorientationMatrix[9];						
			double temp[6];
			double ref[6];
			double vectAx[3];
			double vectAy[3];
			double vectAz[3];
			double vectBx[3];
			double vectBy[3];
			double vectBz[3];
			double MagnCrosProd_A;
			double MagnCrosProd_B;
			double accnorm, magnorm, earthnorm, VectAynorm, VectAznorm, VectBynorm, VectBznorm;
			double m[9], n[9];
			double quat_e[4];
			double invquat_e[4];
			double mult;
			double quat_new[4];
			double quat_old[4];
			double ecs,accex_ADCnumber,y,accey_ADCnumber,z,accez_ADCnumber,mx,magnx_ADCnumber,my,magny_ADCnumber,mz,magnz_ADCnumber;
// SLERP Variables
			double trace, Suca;
			double tol[4], omega, sinom, cosom, scale0, scale1, tez, orientationMatrixA[9];

int main(void) 
{  
	// set up our class: create a class definition  
	setup((t_messlist**) &this_class, (method)triad_new, (method)triad_free, (short)sizeof(t_triad), 0L,A_GIMME, 0); 
    addmess((method)triad_list, "list", A_GIMME, 0);
	addmess((method)triad_assist, "assist", A_CANT, 0);
	finder_addclass("Maths","triad");
	post("....I'm TRIAD Object!....from AHRS Library...",0);
	return 0;
}

/* ------------triad_new --------------*/

void *triad_new(long value) 
{ 
	t_triad *triad;
	triad = (t_triad *)newobject(this_class);	// create the new instance and return a pointer to it 
	triad->m_R4 = floatout(triad);
	triad->m_R3 = floatout(triad);
	triad->m_R2 = floatout(triad);
	triad->m_R1 = floatout(triad);

return(triad);
} 

etc.

etc.