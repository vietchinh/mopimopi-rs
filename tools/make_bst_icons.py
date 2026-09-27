"""Draws the Beastmaster (BST) job icon in every icon set of the overlay.

Source: tools/icons/bst_source.png (the 64x64 game icon, "frame" style). The gold glyph is cut out of it and
re-rendered per set with the colours, outline and size learned from the existing icons of that set
(antique, black, blue, gear, glow, gold, green, orange, purple, red, silver); the frame set uses the
artwork itself with the tile colour matched to the other damage-role frames.

Usage: python3 tools/make_bst_icons.py [--source ICON.png] [output_dir]      needs numpy, scipy, pillow

--source can be a framed icon of any square size (same layout as the 64x64 game icon) or a bare glyph on a
transparent background (then the `frame` set is skipped, because there is no tile to reuse). A larger source
gives cleaner strokes. output_dir defaults to public/images/icon (files are written as <set>/BST.png).
"""
import numpy as np, os, sys
from PIL import Image, ImageFilter
import argparse
ROOT=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON=os.path.join(ROOT,'public/images/icon')
_args=argparse.ArgumentParser(description=__doc__.split('\n')[0])
_args.add_argument('output_dir',nargs='?',default=ICON,help='where <set>/BST.png is written (default: the icon sets of the overlay)')
_args.add_argument('--source',default=os.path.join(ROOT,'tools/icons/bst_source.png'),
                   help='the job icon: a framed icon (any size, square, same layout as the game icon) or a bare glyph on a transparent background')
_opts=_args.parse_args()
SRC=_opts.source
OUT=_opts.output_dir
SS=4  # supersampling for smooth edges

def lum(rgb): return 0.299*rgb[...,0]+0.587*rgb[...,1]+0.114*rgb[...,2]

# ---------- 1. glyph extraction ----------
src_img=Image.open(SRC).convert('RGBA')
if src_img.width!=src_img.height: raise SystemExit('the source icon must be square')
src=np.array(src_img).astype(float)
K=src_img.width/64.0                                   # the layout below is described for a 64 px framed icon
GLYPH_ONLY=(src[...,3]<20).mean()>0.45                 # mostly transparent -> a bare glyph without the tile
print('source',src_img.size,'(bare glyph)' if GLYPH_ONLY else '(framed icon)')
from scipy import ndimage
if GLYPH_ONLY:
    alpha=src[...,3]/255
    fg=src[...,:3]
else:
    # framed icon: gold glyph on a maroon tile. Cut the glyph out by its green channel (the tile has almost none).
    R,G,B=src[...,0],src[...,1],src[...,2]
    alpha=np.clip((G-62)/(140-62),0,1)*(src[...,3]/255)
    alpha=np.clip((alpha-0.10)/0.40,0,1)                # solid strokes, crisp edges
    region=np.zeros_like(alpha); lo,hi=int(round(10*K)),int(round(54*K)); region[lo:hi,lo:hi]=1; alpha*=region
    lab,n=ndimage.label(alpha>0.2,structure=np.ones((3,3)))
    sizes=ndimage.sum(alpha>0.2,lab,range(1,n+1))
    keep=np.isin(lab,[i+1 for i,z in enumerate(sizes) if z>=0.15*max(sizes)])   # the glyph parts, not tile corners
    alpha*=ndimage.binary_dilation(keep,iterations=max(1,int(round(K))))
    bg=np.array([105,48,49.])
    fg=np.clip((src[...,:3]-(1-alpha[...,None])*bg)/np.maximum(alpha[...,None],1e-3),0,255)
ys,xs=np.where(alpha>0.25); x0,x1,y0,y1=xs.min(),xs.max()+1,ys.min(),ys.max()+1
glyph_a=alpha[y0:y1,x0:x1]; glyph_c=fg[y0:y1,x0:x1]           # cropped glyph
print("glyph crop",glyph_a.shape)

def glyph_rgba(max_dim, ss=SS):
    """glyph resized so its longer side is max_dim*ss px; returns (rgb float, alpha float)"""
    h,w=glyph_a.shape; s=max_dim*ss/max(h,w); nh,nw=round(h*s),round(w*s)
    a=np.array(Image.fromarray((glyph_a*255).astype('uint8')).resize((nw,nh),Image.LANCZOS)).astype(float)/255
    c=np.stack([np.array(Image.fromarray(glyph_c[...,i].astype('uint8')).resize((nw,nh),Image.LANCZOS)).astype(float) for i in range(3)],-1)
    return c,a

def place(canvas_size, arr, offset_y=0, offset_x=0, ss=SS):
    """center an (h,w[,c]) supersampled array on a canvas (canvas_size*ss)"""
    H=W=canvas_size*ss; h,w=arr.shape[:2]
    out=np.zeros((H,W)+arr.shape[2:]); y=(H-h)//2+offset_y*ss; x=(W-w)//2+offset_x*ss
    out[y:y+h,x:x+w]=arr; return out

def down(arr,size):
    """supersampled float array (0..255 or 0..1) -> size x size"""
    if arr.ndim==2: return np.array(Image.fromarray(arr.astype('float32')).resize((size,size),Image.LANCZOS))
    return np.stack([down(arr[...,i],size) for i in range(arr.shape[2])],-1)

def to_img(rgb,a):  # rgb 0..255 float (size,size,3), a 0..1
    return Image.fromarray(np.dstack([np.clip(rgb,0,255),np.clip(a,0,1)*255]).round().astype('uint8'),'RGBA')

def blur(a,r):
    return np.array(Image.fromarray((np.clip(a,0,1)*255).astype('uint8')).filter(ImageFilter.GaussianBlur(r))).astype(float)/255
def dilate(a,px):
    im=Image.fromarray((a*255).astype('uint8')); return np.array(im.filter(ImageFilter.MaxFilter(px*2+1))).astype(float)/255
def erode(a,px):
    im=Image.fromarray((a*255).astype('uint8')); return np.array(im.filter(ImageFilter.MinFilter(px*2+1))).astype(float)/255

def thicken(A,C,px):
    '''make strokes ~2*px/SS pixels bolder in target-pixel units; keeps colours by spreading the brightest neighbour'''
    A2=blur(dilate(A,px),0.45*SS); A2=np.maximum(A2,A)
    Cm=(C*(A>0.35)[...,None]).clip(0,255).astype('uint8')
    Cd=np.stack([np.array(Image.fromarray(Cm[...,i]).filter(ImageFilter.MaxFilter(px*2+1))) for i in range(3)],-1).astype(float)
    C2=np.where((A>0.35)[...,None],C,Cd)
    return A2,C2

def load_set(s):
    ims=[np.array(Image.open(f'{ICON}/{s}/{f}').convert('RGBA')).astype(float) for f in sorted(os.listdir(f'{ICON}/{s}')) if f!='BST.png']
    return ims

# ---------- 2. learn colours of a set ----------
def learn_colored(s):
    tops,bots,outl=[],[],[]
    for im in load_set(s):
        a=im[...,3]/255; l=lum(im[...,:3]); m=(a>0.95)&(l>85)
        ys,xs=np.where(m)
        if len(ys)<50: continue
        y0,y1=ys.min(),ys.max(); h=y1-y0+1
        t=m&(np.arange(im.shape[0])[:,None]<y0+0.3*h); b=m&(np.arange(im.shape[0])[:,None]>y0+0.7*h)
        if t.sum()>5 and b.sum()>5: tops.append(im[t][:,:3].mean(0)); bots.append(im[b][:,:3].mean(0))
        d=(a>0.95)&(l<45)
        if d.sum()>10: outl.append(im[d][:,:3].mean(0))
    return np.mean(tops,0),np.mean(bots,0),np.mean(outl,0)

def make_colored(s):
    top,bot,outl=learn_colored(s)
    size=40; c,a=glyph_rgba(31)
    A=place(size,a); C=place(size,c); A,C=thicken(A,C,3)
    h=A.shape[0]; rows=np.where(A.max(1)>0.3)[0]; y0,y1=rows.min(),rows.max()
    t=np.clip((np.arange(h)-y0)/max(y1-y0,1),0,1)[:,None,None]
    fill=top*(1-t)+bot*t                                           # vertical gradient like the existing icons
    detail=(lum(C)-lum(C)[A>0.5].mean())/255                          # bevel from the original artwork
    fill=fill*(1+0.9*detail[...,None]) 
    ow=1.0*SS                                                        # outline ~1.15px
    outer=blur(dilate(A,int(round(ow))),0.3*SS)
    outer=np.maximum(outer,A)
    rgb=fill*A[...,None]+outl*(1-A[...,None])
    # soft dark shadow toward the bottom-right, as in the existing sets
    sh=np.roll(np.roll(outer,int(0.6*SS),0),int(0.3*SS),1); sh=blur(sh,0.5*SS)*0.35
    tot_a=np.maximum(outer,sh)
    rgb=np.where((outer>0.01)[...,None],rgb,outl*0.5)
    return to_img(down(rgb,size),down(tot_a,size))

ANTIQUE_RAMP=[(0.0,(112,82,28)),(0.3,(163,132,65)),(0.6,(195,171,109)),(1.0,(238,220,165))]   # measured from the existing set

def ramp(v,stops):
    xs=[p for p,_ in stops]; out=np.zeros(v.shape+(3,))
    for ch in range(3): out[...,ch]=np.interp(v,xs,[c[ch] for _,c in stops])
    return out

def make_antique():
    size=28; c,a=glyph_rgba(23); A=place(size,a); C=place(size,c); A,C=thicken(A,C,1)
    h=A.shape[0]; rows=np.where(A.max(1)>0.3)[0]; y0,y1=rows.min(),rows.max()
    t=np.clip((np.arange(h)-y0)/max(y1-y0,1),0,1)[:,None]
    detail=(lum(C)-lum(C)[A>0.5].mean())/255
    core=blur(A,1.1*SS)                                        # bright centre line, darker edges
    edge=A-erode(A,int(0.7*SS))
    shade=np.clip(0.30+0.55*core+0.5*detail+0.18*(1-t)-0.35*edge,0,1)   # lit from the top, darker rim
    fill=ramp(shade,ANTIQUE_RAMP)
    dark=np.array([66.,44.,16.])
    outer=np.maximum(blur(dilate(A,int(0.8*SS)),0.3*SS),A)
    sh=blur(np.roll(A,int(1.0*SS),0),0.8*SS)*0.5
    rgb=fill*A[...,None]+dark*(1-A[...,None])
    return to_img(down(rgb,size),down(np.maximum(outer,sh),size))

def make_frame():
    im=Image.open(SRC).convert('RGBA').resize((32,32),Image.LANCZOS); a=np.array(im).astype(float)
    # tile colour of the existing damage-role frames (MNK, PCT, VPR, DRG, ...) vs the source tile
    tiles=[]
    for j in ['MNK','PCT','VPR','DRG','SAM','NIN','BLM','MCH']:
        f=np.array(Image.open(f'{ICON}/frame/{j}.png').convert('RGBA')).astype(float)
        m=(f[...,3]>250)&(f[...,0]>f[...,1]*1.25)&(lum(f[...,:3])<95)&(lum(f[...,:3])>25); tiles.append(f[m][:,:3].mean(0))
    target=np.mean(tiles,0)
    m=(a[...,3]>250)&(a[...,0]>a[...,1]*1.5)&(a[...,1]<95)
    cur=a[m][:,:3].mean(0); k=target/cur
    a[m,:3]=np.clip(a[m,:3]*k,0,255)                                        # only the tile, not the gold
    print('frame tile',cur.round(),'->',target.round())
    return Image.fromarray(a.round().astype('uint8'),'RGBA')

def make_glow():
    size=32
    core_col=np.array([255,250,232.]); rim_col=np.array([255,214,120.]); glow_col=np.array([255,158,20.])
    c,a=glyph_rgba(24); A=place(size,a); A,_=thicken(A,place(size,c),3)
    inner=erode(A,int(0.8*SS))
    glow=np.maximum(blur(A,1.6*SS),0)*1.5
    glow=np.clip(glow,0,1)
    g2=np.clip(blur(A,3.2*SS)*1.2,0,1)
    glow_a=np.clip(np.maximum(glow*0.85,g2*0.6),0,1)
    core_a=A
    rgb=np.zeros(A.shape+(3,))
    rgb[:]=glow_col
    rim=core_a[...,None]*rim_col + (1-core_a[...,None])*glow_col
    rgb=np.where((core_a>0.02)[...,None],rim,rgb)
    rgb=np.where((inner>0.02)[...,None],core_col*inner[...,None]+rim*(1-inner[...,None]),rgb)
    tot=np.clip(np.maximum(glow_a,core_a),0,1)
    return to_img(down(rgb,size),down(tot,size))

def clean_gear_tile():
    ims=np.stack(load_set('gear'),0)
    return np.median(ims,0)              # per-pixel median removes the glyphs -> clean tile

def make_gear():
    """Dark engraved glyph on the light stone tile. The strokes are kept thin so the shield's inner
    shapes stay open (like WAR / VPR), and coverage is sharpened at the final size so they look crisp."""
    size=40; tile=clean_gear_tile()
    tops,bots=[],[]
    for im in load_set('gear'):
        d=(lum(im[...,:3])<lum(tile[...,:3])-70)&(im[...,3]>240)
        ys,xs=np.where(d)
        if len(ys)<40: continue
        y0,y1=ys.min(),ys.max(); h=y1-y0+1
        t=d&(np.arange(40)[:,None]<y0+0.35*h); b=d&(np.arange(40)[:,None]>y0+0.65*h)
        if t.sum()>4 and b.sum()>4: tops.append(im[t][:,:3].mean(0)); bots.append(im[b][:,:3].mean(0))
    top,bot=np.mean(tops,0),np.mean(bots,0)
    c,a=glyph_rgba(36); A=place(size,a)
    At=down(A,size)                                              # coverage at the real icon size, no thickening
    At=blur_small(At,0.55)                                        # merge the source's fine 1px hatching into a smooth shade
    At=np.clip((At-0.18)/0.42,0,1)                              # sharpen: thin strokes become solid dark lines
    rows=np.where(At.max(1)>0.3)[0]; y0,y1=rows.min(),rows.max()
    t=np.clip((np.arange(size)-y0)/max(y1-y0,1),0,1)[:,None,None]
    fill=(top*(1-t)+bot*t)*0.82
    hi=blur_small(np.roll(At,1,0),0.6)*(1-At)                   # light lip below the glyph
    rgb=tile[...,:3]*(1-At[...,None])+fill*At[...,None]
    rgb=rgb+(255-rgb)*(0.40*hi[...,None])
    return to_img(rgb,tile[...,3]/255)

def blur_small(a,r):
    return np.array(Image.fromarray((np.clip(a,0,1)*255).astype('uint8')).filter(ImageFilter.GaussianBlur(r))).astype(float)/255

os.makedirs(OUT,exist_ok=True)
sets=sorted(os.listdir(ICON))
for s in sets:
    if s=='frame' and GLYPH_ONLY:
        print('frame: skipped (a bare glyph has no tile; keep or supply the framed icon)'); continue
    if s=='antique': im=make_antique()
    elif s=='frame': im=make_frame()
    elif s=='glow': im=make_glow()
    elif s=='gear': im=make_gear()
    else: im=make_colored(s)
    os.makedirs(f'{OUT}/{s}',exist_ok=True); im.save(f'{OUT}/{s}/BST.png'); print(s,im.size)

# contact sheet: existing WAR, MNK, PCT, VPR next to the new BST, 4x
S=4; cell=44*S; jobs=['WAR','MNK','PCT','VPR','BST']
sheet=Image.new('RGBA',(len(jobs)*cell+90,len(sets)*cell),(60,60,60,255))
from PIL import ImageDraw
d=ImageDraw.Draw(sheet)
for r,s in enumerate(sets):
    d.text((4,r*cell+cell//2),s,fill=(255,255,255,255))
    for c_,j in enumerate(jobs):
        p=f'{OUT}/{s}/BST.png' if j=='BST' else f'{ICON}/{s}/{j}.png'
        if j=='BST' and not os.path.exists(p): p=f'{ICON}/{s}/BST.png'      # skipped set: show the installed icon
        im=Image.open(p).convert('RGBA'); im=im.resize((im.width*S,im.height*S),Image.NEAREST)
        sheet.alpha_composite(im,(90+c_*cell,r*cell+(cell-im.height)//2))
sheet.save(os.path.join(os.path.dirname(OUT.rstrip('/')),'bst_preview.png') if OUT!=ICON else '/tmp/bst_preview.png'); print('preview sheet written')
