import flask
import form
import json
import glob
import platform
import os,sys
import argparse
import time
import string
if getattr(sys,'frozen',False):
    #running in bundle
    work_dir = sys._MEIPASS
else:
    work_dir = os.path.dirname(os.path.abspath(__file__))

template_dir=os.path.join(work_dir,'templates')
app = flask.Flask(__name__,
                  template_folder=template_dir)
def record_hymn_usage(h_num):
    record_file=os.path.expanduser("~/Desktop/HymnUsage.txt")
    try:
        lines=open(record_file).readlines()
    except FileNotFoundError:
        lines=[]
    use_line = time.strftime(f"%b-%d-%Y H{h_num}\n")
    if use_line not in lines:
        lines.append(use_line)
    with open(record_file,"w") as of:
        of.writelines(lines)
def update_liturgy(current_liturgy,update_request):
    import re
    if 'psalm_number' in update_request.args:
        ph = 'P'
        song = 'Psalm '+ update_request.args.get('psalm_number')
    elif 'hymn_number' in update_request.args:
        ph = 'H'
        song_num = int(update_request.args.get('hymn_number'))
        song = f'Hymn {song_num}'
        if song_num in (38,50,66,79):
            record_hymn_usage(song_num)
    else:
        return
    
    liturgy_line= { 'song' : song,
                    'verses' : []
                    }
    MAX_VERSES=66
    for v in range(MAX_VERSES+1):
        k = f"{ph}V{v}"
        if k in update_request.args and update_request.args.get(k) == 'on':
            liturgy_line['verses'].append(str(v))
    current_liturgy.append(liturgy_line)

def get_liturgy():
    try:
        liturgy = json.loads(flask.request.cookies.get('liturgy'))
    except TypeError:
        liturgy = []
    return liturgy
@app.route("/")
def home():
    current_liturgy = get_liturgy()

    photo_list=[]
    for l in current_liturgy:
        song_dir = ("".join(l['song'].split())).lower()
        for i,v in enumerate(l['verses']):
            
            verses_before = [_ for _ in l['verses'][:i] ]
            verses_after = [_ for _ in l['verses'][i+1:]]
            verse_list = sorted(glob.glob(work_dir+'/photos/'+song_dir+f"/{v}[a-z].png") +
                                glob.glob(work_dir+'/photos/'+song_dir+f"/{v}.png"))
            verse_list = [ v[len(work_dir):] for v in verse_list]
            for vl in verse_list:
                current_verse = os.path.splitext(os.path.basename(vl))[0]
                photo_list.append({"title":f"{l['song']}: ",
                                   "verselistbefore":" ".join(verses_before),
                                   "verselistcurrent":current_verse,
                                   "verselistafter":" ".join(verses_after),
                                   "path":vl})
    resp = flask.make_response(flask.render_template('form.html',
                                                     liturgy=current_liturgy,
                                                     photo_list=photo_list))
    
    return resp
@app.route("/add_song")
def add_song():
    current_liturgy = get_liturgy()
    update_liturgy(current_liturgy,flask.request)
    resp = flask.make_response(flask.redirect('/'))
    lit_json = json.dumps(current_liturgy)

    resp.set_cookie('liturgy',lit_json)
    return resp
@app.route("/clear_liturgy")
def clear_liturgy():
    resp = flask.make_response(flask.redirect('/'))
    lit_json = json.dumps([])
    resp.set_cookie('liturgy',lit_json)
    return resp
@app.route('/display')
def display():
    current_liturgy = get_liturgy()
    resp = flask.make_response(flask.render_template('display.html',
                                                     liturgy=current_liturgy,
                                                     photo_list=photo_list))
    return resp
@app.route('/_photos/<path:path>')
def photo_serve(path):
    return flask.send_from_directory('photos', path)
@app.route('/jquery.min.js')
def jquery_serve():
    return flask.send_file("jquery.min.js")

@app.route('/photos/<path:path>')
def cropped_photo_serve(path):
    import photo,io
    img = photo.crop_to_content(path)
    #return flask.send_file(
    #    io.BytesIO(img),
    #    mimetype="image/png")
        
    response = flask.make_response(img)
    response.headers.set('Content-Type', 'image/png')
    #response.headers.set(
    #    'Content-Disposition', 'attachment', filename=f'photos/{path}.jpg' )
    return response
    

@app.route("/verses.json")
def verses_json():
    d = {
        'hymns':form.hymn_array,
        'psalms':form.psalm_array
        }
    return json.dumps(d)

PORT=5001
def runwebview():
    import time
    import webbrowser
    time.sleep(3)
    webbrowser.open_new_tab('http://localhost:{}'.format(PORT))

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--no-window',action='store_true')
    args = parser.parse_args()
    print(args)
    if not args.no_window:
        import threading
        t=threading.Thread(target=runwebview)
        t.start()        
    try:
        app.run(host='0.0.0.0',port=PORT)
    except OSError:
        t.join()
