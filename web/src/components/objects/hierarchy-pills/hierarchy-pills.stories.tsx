import React from 'react';
import { Story, Meta } from '@storybook/react';

import { Exocore, ExocoreInstance } from 'exocore';
import { registerTypes } from '../../../exocore';

import '../../../style/style.less';
import { HierarchyPills, IProps } from './hierarchy-pills';
import { EntityParent } from '../../../store/collections';
import { exomind } from '../../../protos';
import { TraitIcon } from '../../../store/entities';

Exocore.default = new ExocoreInstance(null, null);
registerTypes(Exocore.default);


export default {
    title: 'Objects/HierarchyPills',
    component: HierarchyPills,
} as Meta;

const Template: Story<IProps> = (args) => <HierarchyPills {...args} />;

const getCol = (icon: TraitIcon, name: string, parents?: EntityParent[]): EntityParent => {
    let minParent: EntityParent;
    if (parents && parents.length > 0) {
        minParent = parents[0];
    }

    return {
        entityId: name,
        icon,
        name,
        collection: new exomind.base.Collection({ name }),
        parents,
        minParent,
    }
};

export const Pill = Template.bind({});
Pill.args = {
    collections: [
        getCol({ fa: 'star' }, 'col1'),
        getCol({ emoji: '😬' }, 'col2'),
        getCol({ fa: 'folder-o' }, 'col3', [getCol({ fa: 'folder-o' }, 'col3', [getCol({ emoji: '📥' }, 'parent')])]),
        getCol({ emoji: '😬' }, 'child long long', [getCol({ emoji: '🎤' }, 'parent long long', [getCol({ emoji: '📥' }, 'grand parent long long')])]),
    ]
};