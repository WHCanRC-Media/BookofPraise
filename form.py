
import glob
import os
import sys
N_PSALMS=150
N_HYMNS=85
psalm_array=[[] for _ in range(N_PSALMS)]
if getattr(sys,'frozen',False):
    #running in bundle
    work_dir = sys._MEIPASS
else:
    work_dir = os.path.dirname(__file__)
for i,p in enumerate(psalm_array):
    for v in range(70):#at least 66 (psalm119)
        if glob.glob(f'{work_dir}/photos/psalm{i+1}/{v}.png'):
            p.append(v)
        elif glob.glob(f'{work_dir}/photos/psalm{i+1}/{v}[a-z].png'):
            p.append(v)
hymn_array=[[] for _ in range(N_HYMNS)]
for i,p in enumerate(hymn_array):
    for v in range(70):#at least 66 (psalm119)
        if glob.glob(f'{work_dir}/photos/hymn{i+1}/{v}.png'):
            p.append(v)
        elif glob.glob(f'{work_dir}/photos/hymn{i+1}/{v}[a-z].png'):
            p.append(v)
