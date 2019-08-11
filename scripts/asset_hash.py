import json
import os
from hashlib import sha1

def file_hash(file_name):
    m = sha1()
    with open(file_name, 'rb') as f:
        contents = f.read()
        m.update(contents)
    return m.hexdigest(), contents

def main(assets_json):
    with open(assets_json, 'r') as f:
        assets = json.loads(f.read())

    changed = False

    for uri, asset in assets.items():
        path, filename = os.path.split(asset["src"])
        name, file_ext = os.path.splitext(filename)

        hash_digest, contents = file_hash(asset["src"])
        hash_digest = hash_digest[:10]

        target_file = "{}/{}.{}{}".format(
                path, name, hash_digest, file_ext)

        prefix, ext = os.path.splitext(uri)
        target = "{}.{}{}".format(prefix, hash_digest, ext)

        changed = changed or target != asset['target']

        asset['target'] = target
        with open(target_file, 'wb') as f:
            f.write(contents)

    if changed:
        with open(assets_json, 'w') as f:
            f.write(json.dumps(assets))

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='Append contents hash to filename')
    parser.add_argument('assets_json_path', type=str, nargs=1,
                               help='path to assets.json')
    args = parser.parse_args()
    main(args.assets_json_path[0])

