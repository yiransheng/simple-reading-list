import React from 'react';
import {Bookmark, Callback} from '../interface';
import {Input} from './Input';
import {TextArea} from './TextArea';

export interface Props {
  bookmark: Bookmark;

  onUpdate: Callback<Bookmark>;

  onSubmit: Callback<void>;
}

function parseTags(raw: string): string[] {
  const tags = [];
  const word = /[a-zA-Z0-9]+/g;
  let result;
  // tslint:disable-next-line:no-cond-assign
  while ((result = word.exec(raw))) {
    tags.push(result[0]);
  }

  return tags;
}

export function BookmarkForm(props: Props): JSX.Element {
  const {bookmark, onUpdate, onSubmit} = props;

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onSubmit();
  };

  const update = function(par: Partial<Bookmark>) {
    onUpdate({...bookmark, ...par});
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <Input
          controlled
          label="title"
          placeholder="Link title"
          value={bookmark.title}
          onChange={title => update({title})}
        />
        <Input
          controlled
          label="url"
          placeholder="Link url"
          value={bookmark.url}
          onChange={url => update({url})}
        />
        <TextArea
          label="description"
          value={bookmark.body}
          onChange={body => update({body})}
        />
        <Input
          controlled
          label="tags"
          value={bookmark.tags.join(', ')}
          onChange={tags => update({tags: parseTags(tags)})}
        />
        <button type="submit">Post</button>
      </form>
    </div>
  );
}
