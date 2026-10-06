import os
import subprocess

root = r'C:\Users\Administrator\Desktop\ccc\wanzhen'

BS = b'\\'
rep = [
    (b'C:' + BS + b'Users' + BS + b'ADMINI~1' + BS + b'Desktop' + BS + b'ccc' + BS + b'trees' + BS + b'wanzhen', b'<trees-root>'),
    (b'C:/Users/ADMINI~1/Desktop/ccc/trees/wanzhen', b'<trees-root>'),
    (b'C:' + BS + b'Users' + BS + b'Administrator' + BS + b'Desktop' + BS + b'ccc' + BS + b'trees' + BS + b'wanzhen', b'<trees-root>'),
    (b'C:/Users/Administrator/Desktop/ccc/trees/wanzhen', b'<trees-root>'),
    (b'C:' + BS + b'Users' + BS + b'ADMINI~1' + BS + b'Desktop' + BS + b'ccc' + BS + b'wanzhen', b'<repo>'),
    (b'C:/Users/ADMINI~1/Desktop/ccc/wanzhen', b'<repo>'),
    (b'C:' + BS + b'Users' + BS + b'Administrator' + BS + b'Desktop' + BS + b'ccc' + BS + b'wanzhen', b'<repo>'),
    (b'C:/Users/Administrator/Desktop/ccc/wanzhen', b'<repo>'),
    (b'C:' + BS + b'Users' + BS + b'ADMINI~1' + BS + b'Desktop' + BS + b'ccc' + BS + b'bevy-ai-workflow', b'<workflow-repo>'),
    (b'C:' + BS + b'Users' + BS + b'ADMINI~1', b'<home>'),
    (b'C:' + BS + b'Users' + BS + b'Administrator', b'<home>'),
    (b'D:' + BS + b'software' + BS + b'Sysinternals', b'<sysinternals>'),
    (b'D:' + BS + b'software', b'<d-drive>' + BS + b'software'),
    (b'D:' + BS, b'<d-drive>' + BS),
]
pats = [
    'C:' + chr(92) + 'Users' + chr(92) + 'ADMINI~1',
    'C:' + chr(92) + 'Users' + chr(92) + 'Administrator',
    'C:/Users/ADMINI~1',
    'C:/Users/Administrator',
    'D:' + chr(92) + 'software',
    'D:' + chr(92),
]
hit = set()
for pat in pats:
    out = subprocess.run(['grep', '-rIlF', pat, 'docs/'], capture_output=True, text=True, cwd=root).stdout.split()
    hit.update(out)
n = 0
for f in sorted(hit):
    p = os.path.join(root, f)
    b = open(p, 'rb').read()
    o = b
    for a, c in rep:
        b = b.replace(a, c)
    if b != o:
        open(p, 'wb').write(b)
        n += 1
        print('cleaned:', f)
print('files cleaned:', n)
