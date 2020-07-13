import cv2
import numpy as np
import sys

def get_resized_image(img_filename):
    img = cv2.imread(img_filename)
    ideal_aspect_ratio = 4/3
    if img.shape[1] /img.shape[0] < ideal_aspect_ratio:
        # to tall,widen
        new_width = img.shape[0]*4//3
    
        new_img = np.zeros((img.shape[0],new_width,3),dtype=np.uint8)+0xFF
        
        side_pad = (new_width-img.shape[1])//2
    
        new_img[:,side_pad:side_pad+img.shape[1],:] = img
    else:
        #to wide,heighten
        new_height = img.shape[1]*3//4
        new_img = np.zeros((new_height,img.shape[1],3),dtype=np.uint8)+0xff
        
        pad = (new_height-img.shape[0])//2
    
        new_img[pad:pad+img.shape[0],:,:] = img
    
    new_img=cv2.resize(new_img,(640*4,480*4))
    #add 1px border
    new_img[:,-1,:]=0
    new_img[:,0,:]=0
    new_img[-1,:,:]=0
    new_img[0,:,:]=0
    #invert black/white
    #new_img=0xFF-new_img
    buf =cv2.imencode('.png',new_img)
    return buf[1].flatten().tobytes()


if __name__ == '__main__':
    filename = 'photos/psalm1/'+sys.argv[1]+'.png'
    with open('out.png',"wb") as f:
        b = get_resized_image(filename)
        f.write(b)
