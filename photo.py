import cv2
import numpy as np
import sys
if getattr(sys,'frozen',False):
    #running in bundle
    work_dir = sys._MEIPASS
else:
    work_dir = os.path.dirname(__file__)

def crop_to_content(filename):
    img_filename=work_dir+"/photos/"+filename
    img = cv2.imread(img_filename)
    top_row=0
    bot_row=-1
    for r in range(img.shape[1]):
        if np.min(img[r]) < 200:
            top_row=r
            break;
    for r in range(1,img.shape[1]):
        if np.min(img[-r]) < 200:
            bot_row=-r
            break
    first_col = 0
    last_col = -1
    for c in range(img.shape[0]):
        if np.min(img[:,c]) < 200:
            first_col=c
            break
    for c in range(1,img.shape[0]):
        if np.min(img[:,-c]) < 200:
            last_col=-c
            break
    last_col=min(-1,last_col+2)
    bot_row=min(-1,bot_row+2)
    first_col = max(0,first_col-2)
    top_row = max(0,top_row-2)

    new_img = img[top_row:bot_row,first_col:last_col]
            
    ret,buf =cv2.imencode('.png',new_img)
    assert ret
    return buf.tobytes()
    
