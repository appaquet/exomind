import React from 'react';
import { Story, Meta } from '@storybook/react';

import { IProps, EntityList } from './entity-list';
import { Exocore, exocore, ExocoreInstance, toProtoTimestamp } from 'exocore';
import { exomind } from '../../../protos';
import { registerTypes } from '../../../exocore';
import '../../../style/style.less';
import { ButtonAction, EntityActions } from './entity-action';
import { EntityTraits } from '../../../store/entities';

Exocore.default = new ExocoreInstance(null, null);
registerTypes(Exocore.default);


export default {
    title: 'Objects/EntityList',
    component: EntityList,
} as Meta;

const Template: Story<IProps> = (args) => <EntityList {...args} />;

export const List = Template.bind({});
List.args = {
    entities: [
        new exocore.store.Entity({
            id: '1',
            traits: [
                new exocore.store.Trait({
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(new exomind.base.Note({
                        title: "hello world",
                    })),
                })
            ]
        }),
        new exocore.store.Entity({
            id: '2',
            traits: [
                new exocore.store.Trait({
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(new exomind.base.Link({
                        title: "Some link",
                        url: "https://www.google.com",
                    })),
                })
            ]
        }),
        new exocore.store.Entity({
            id: '3',
            traits: [
                new exocore.store.Trait({
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(new exomind.base.EmailThread({
                        subject: 'Some subject',
                        from: new exomind.base.Contact({ name: 'Some name' }),
                        snippet: 'Some snippet'
                    })),
                })
            ]
        }),
        new exocore.store.Entity({
            id: '4',
            traits: [
                new exocore.store.Trait({
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(new exomind.base.Task({
                        title: 'Some task',
                    })),
                })
            ]
        }),
        new exocore.store.Entity({
            id: '5',
            traits: [
                new exocore.store.Trait({
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(new exomind.base.Collection({
                        name: '😬 Collection name'
                    })),
                })
            ]
        }),
    ],
    actionsForEntity: (et: EntityTraits) => {
        const actions = new EntityActions([
            new ButtonAction('check', () => 'remove'),
            new ButtonAction('clock-o', () => null)
        ]);

        if (et.id == '4') {
            actions.buttons.push(
                new ButtonAction('folder-open-o', () => 'remove'),
                new ButtonAction('inbox', () => null),
                new ButtonAction('thumb-tack', () => null)
            );
        }

        return actions;
    },
};