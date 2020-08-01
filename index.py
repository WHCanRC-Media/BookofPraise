import flask
import form
import json
import glob
app = flask.Flask(__name__)

def update_liturgy(current_liturgy,update_request):
    import re
    if 'psalm_number' in update_request.args:
        ph = 'P'
        song = 'Psalm '+ update_request.args.get('psalm_number')
    elif 'hymn_number' in update_request.args:
        ph = 'H'        
        song = 'Hymn '+update_request.args.get('hymn_number')
    else:
        return

    liturgy_line= { 'song' : song,
                    'verses' : []
                    }
    for v in range(66):
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
        for v in l['verses']:
            verse_list = sorted(glob.glob('photos/'+song_dir+f"/{v}[a-z].png") +
                                glob.glob('photos/'+song_dir+f"/{v}.png"))
            
            for vl in verse_list:                
                photo_list.append({"title":l['song']+f": {v}",
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
@app.route('/photos/<path:path>')
def photo_serve(path):
    return flask.send_from_directory('photos', path)
@app.route("/verses.json")
def verses_json():
    d = {
        'hymns':form.hymn_array,
        'psalms':form.psalm_array
        }
    return json.dumps(d)
if __name__ == "__main__":
    app.run(host="0.0.0.0",port=5000,debug=True) 
