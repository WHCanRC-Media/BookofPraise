
import glob
import os.path
N_PSALMS=150
N_HYMNS=84
psalm_array=[[] for _ in range(N_PSALMS)]
for i,p in enumerate(psalm_array):
    for v in range(70):#at least 66 (psalm119)
        if glob.glob(f'photos/psalm{i+1}/{v}.png'):
            p.append(v)
        elif glob.glob(f'photos/psalm{i+1}/{v}[a-z].png'):
            p.append(v)

hymn_array=[[] for _ in range(N_HYMNS)]
for i,p in enumerate(hymn_array):
    for v in range(70):#at least 66 (psalm119)
        if glob.glob(f'photos/hymn{i+1}/{v}.png'):
            p.append(v)
        elif glob.glob(f'photos/hymn{i+1}/{v}[a-z].png'):
            p.append(v)

#def form():
#    psalms =[os.path.split(p)[-1] for p in glob.glob('photos/psalm*')]
#    rows=[]
#    for p in sorted(psalms):
#        verses = [os.path.split(v)[-1] for v in glob.glob(f'photos/{p}/*')]
#        verses = sorted(verses)
#        verses = [os.path.splitext(v)[0] for v in verses]
#        cboxes = []
#        for v in verses:        
#            cboxes.append(f'<label for="{p}">V{v}</label>'+
#                          f'<input type="checkbox" id="{p}V{v}" name="{p}V{v}">')
#        row = "<tr><td>{}</td><td>{}</td><td>".format(p,"".join(cboxes))
#        rows.append(row)
#    return ("<form action=/page>"+
#            '<input type="submit" value="Submit">'
#            "<table>\n"+
#            "\n".join(rows)+
#            "\n</table>"+
#
#            "</form>")

