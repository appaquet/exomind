import classNames from 'classnames';
import { exocore } from 'exocore';
import { memoize } from 'lodash';
import * as React from 'react';
import EmailFlows from '../../../logic/emails-logic';
import { exomind } from '../../../protos';
import { ICollection, Parents } from '../../../store/collections';
import { EntityTrait, EntityTraits } from '../../../store/entities';
import DateUtil from '../../../utils/date-util';
import DragAndDrop from '../../interaction/drag-and-drop/drag-and-drop';
import EditableText from '../../interaction/editable-text/editable-text';
import EntityIcon from '../entity-icon';
import { HierarchyPills } from '../hierarchy-pills/hierarchy-pills';
import { SelectedItem, Selection } from "./selection";
import { EntityActions } from './entity-action';
import { observer } from 'mobx-react';
import { Stores, StoresContext } from '../../../store/stores';
import './entity.less';

export type DropEffect = ('move' | 'copy');

export interface IProps {
    entity: exocore.store.IEntity;
    parentEntity?: exocore.store.IEntity;

    selected?: boolean;
    onSelectionChange?: (sel: Selection) => void;
    onClick?: (e: React.MouseEvent) => void;
    actionsForEntity?: (entity: EntityTraits) => EntityActions;

    draggable?: boolean;
    droppable?: boolean;
    onDropOut?: (droppedEntity: exocore.store.IEntity, effect: DropEffect, droppedEntityParent: exocore.store.IEntity) => void;
    onDropIn?: (droppedEntity: exocore.store.IEntity, effect: DropEffect, droppedEntityParent: exocore.store.IEntity) => void;

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactFragment;
}

interface IState {
    removed: boolean;
    selected: boolean;
    hovered: boolean;
    beingDragged: boolean;
    parents?: Parents,
}

export class Entity extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);

        this.state = {
            removed: false,
            selected: props.selected ?? false,
            hovered: false,
            beingDragged: false,
        };
    }

    render(): React.ReactNode {
        const classes = classNames({
            item: true,
            done: this.state.removed,
            selected: (this.props.selected === true),
            hover: this.state.hovered
        });

        const entityTraits = this.getEntityTraits(this.props.entity);
        let actionsComponent = null;
        let actions;
        if (this.props.actionsForEntity) {
            actions = this.props.actionsForEntity(entityTraits);
            if (this.state.hovered && !actions.isEmpty) {
                actionsComponent = this.renderActions(actions);
            }
        }

        if (!actions) {
            actions = new EntityActions();
        }

        return (
            <li className={classes}
                onClick={this.handleItemClick}
                onMouseEnter={this.handleItemMouseEnter}
                onMouseLeave={this.handleItemMouseLeave}>

                <div className="swipe-container">
                    <DragAndDrop
                        object={this.props.entity}
                        parentObject={this.props.parentEntity}
                        draggable={this.props.draggable}
                        droppable={this.props.droppable}
                        onDropIn={this.props.onDropIn}
                        onDropOut={this.props.onDropOut}>

                        {actionsComponent}
                        {this.renderElement(entityTraits, actions)}
                    </DragAndDrop>
                </div>
            </li>
        );
    }

    private getEntityTraits = memoize((entity: exocore.store.IEntity) => new EntityTraits(entity));

    private renderActions(actions: EntityActions): React.ReactNode {
        const actionsComponents = actions.buttons.map((action) => {
            const classes = classNames({
                'action-icon': true,
                fa: true,
                ['fa-' + action.icon]: true
            });
            const cb = (e: React.MouseEvent) => {
                const result = action.trigger(e);
                e.stopPropagation();

                if (result == 'remove') {
                    this.removeItem();
                }
            };
            return (
                <li key={action.icon} onClick={cb}>
                    <span className={classes} />
                </li>
            );
        });

        return (
            <div className="item-actions-container column-tooltip">
                <div className="item-actions column-tooltip">
                    <ul>{actionsComponents}</ul>
                </div>
            </div>
        );
    }

    private renderElement(entityTraits: EntityTraits, actions: EntityActions): React.ReactNode {
        return entityTraits.priorityMatch({
            emailThread: (entityTrait) => {
                return this.renderEmailThreadElement(entityTraits, entityTrait);
            },
            draftEmail: (entityTrait) => {
                return this.renderDraftEmailElement(entityTrait);
            },
            email: (entityTrait) => {
                return this.renderEmailElement(entityTrait);
            },
            collection: (entityTrait) => {
                return this.renderCollectionElement(entityTrait);
            },
            task: (entityTrait) => {
                return this.renderTaskElement(actions, entityTrait);
            },
            note: (entityTrait) => {
                return this.renderNoteElement(entityTrait);
            },
            link: (entityTrait) => {
                return this.renderLinkElement(entityTrait);
            },
            default: () => {
                const firstTraitId = entityTraits.entity.traits[0].id;
                const entityTrait = entityTraits.trait(firstTraitId);
                return this.renderDefaultElement(entityTrait);
            }
        });
    }

    private renderEmailThreadElement(entityTraits: EntityTraits, entityTrait: EntityTrait<exomind.base.IEmailThread>): React.ReactNode {
        const thread = entityTrait.message;
        const emails = entityTraits.traitsOfType<exomind.base.IEmail>(exomind.base.Email);

        let title1Markup;
        let title2Markup;
        const subject = thread.subject ?? '(No subject)';
        if (thread.from) {
            const nbEmailsMarkup = (emails.length > 1) ? ` (${emails.length})` : '';
            title1Markup = <div className="title1"><span
                className="name">{EmailFlows.formatContact(thread.from)}{nbEmailsMarkup}</span></div>;
            title2Markup = <div className="title2"><span className="subject">{subject}</span></div>;
        } else {
            title1Markup = <div className="title1"><span className="subject">{subject}</span></div>;
        }

        let snippetMarkup;
        if (thread.snippet != null) {
            snippetMarkup = <div className="text">{thread.snippet}</div>
        }

        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'email-thread': true,
            unread: !thread.read,
        });

        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    {title1Markup}
                    {title2Markup}
                    {snippetMarkup}
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderDraftEmailElement(entityTrait: EntityTrait<exomind.base.IDraftEmail>): React.ReactNode {
        const draft = entityTrait.message;

        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            email: true,
            unread: false
        });
        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">Me</span></div>
                    <div className="title2">{draft.subject ?? '(No subject)'}</div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderEmailElement(entityTrait: EntityTrait<exomind.base.IEmail>): React.ReactNode {
        const email = entityTrait.message;

        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            email: true,
            unread: false,
        });
        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{EmailFlows.formatContact(email.from)}</span></div>
                    <div className="title2">{email.subject ?? '(No subject)'}</div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderCollectionElement(entityTrait: EntityTrait<exomind.base.ICollection>): React.ReactNode {
        return (
            <div className="item-container with-picture collection">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{entityTrait.displayName}</span></div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderTaskElement(actions: EntityActions, entityTrait: EntityTrait<exomind.base.ITask>): React.ReactNode {
        const task = entityTrait.message;
        const onTitleChange = (newTitle: string) => {
            this.handleTaskChange(entityTrait, actions, newTitle);
        };

        return (
            <div className="task item-container with-picture">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1">
                        <EditableText
                            text={task.title}
                            initializeEditing={!!actions.inlineEdit}
                            onChange={onTitleChange}
                        />
                    </div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderNoteElement(entityTrait: EntityTrait<exomind.base.INote>): React.ReactNode {
        const note = entityTrait.message;

        return (
            <div className="note item-container with-picture">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{note.title}</span></div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderLinkElement(entityTrait: EntityTrait<exomind.base.ILink>): React.ReactNode {
        const link = entityTrait.message;

        return (
            <div className="link item-container with-picture">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    {entityTrait.displayName && <div className="title1">{entityTrait.displayName}</div>}
                    <div className="text">{link.url}</div>
                    {this.renderParents(entityTrait.et)}
                </div>
            </div>
        );
    }

    private renderParents(entity: EntityTraits): React.ReactNode {
        return <EntityParents entity={entity} parentEntity={this.props.parentEntity} onSelectionChange={this.props.onSelectionChange} />;
    }

    private renderDefaultElement(entityTrait: EntityTrait<unknown>): React.ReactNode {
        return (
            <div className="default item-container with-picture">
                <div className="date">{this.entityDate(entityTrait)}</div>
                {this.renderEntityImage(entityTrait)}
                <div className="content">
                    <div className="title1">Unknown entity {entityTrait.et.id}</div>
                </div>
            </div>
        );
    }

    private renderEntityImage(entityTrait: EntityTrait<unknown>): React.ReactNode {
        const bubbleClasses = classNames({
            bubble: true,
            ['object-color-' + entityTrait.constants.color]: true
        });

        return <div className="picture">
            <div className={bubbleClasses}>
                <EntityIcon trait={entityTrait} />
            </div>
        </div>;
    }

    private entityDate(entityTrait: EntityTrait<unknown>): React.ReactFragment {
        if (this.props.renderEntityDate) {
            return this.props.renderEntityDate(entityTrait);
        } else {
            return DateUtil.toShortFormat(entityTrait.modificationDate ?? entityTrait.creationDate);
        }
    }

    private removeItem(): void {
        this.setState({
            removed: true
        });
    }

    private handleItemClick = (e: React.MouseEvent): void => {
        if (this.props.onClick) {
            this.props.onClick(e);
        }
    }

    private handleItemMouseEnter = (): void => {
        this.setState({
            hovered: true
        });
    }

    private handleItemMouseLeave = (): void => {
        this.setState({
            hovered: false
        });
    }

    private handleTaskChange(task: EntityTrait<exomind.base.ITask>, actions: EntityActions, newTitle: string): void {
        task.rename(newTitle);

        if (actions.inlineEdit) {
            actions.inlineEdit.trigger();
        }
    }
}

interface EntityParentsProps {
    entity: EntityTraits;
    parentEntity?: exocore.store.IEntity;
    onSelectionChange?: (sel: Selection) => void;
}

interface EntityParentsState {
    parents?: Parents,
}

@observer
class EntityParents extends React.Component<EntityParentsProps, EntityParentsState> {
    static contextType = StoresContext;
    context: Stores;

    constructor(props: EntityParentsProps) {
        super(props);

        this.state = {
        };
    }

    render(): React.ReactNode {
        const parents = this.context.collections.getParents(this.props.entity);

        if (!parents) {
            return <span className="loading"></span>;
        }

        const collections = parents.get().filter((col) => {
            return col.entityId != this.props.parentEntity?.id;
        })

        const onClick = (e: React.MouseEvent, col: ICollection) => {
            if (this.props.onSelectionChange) {
                const item = SelectedItem.fromEntityId(col.entityId);
                this.props.onSelectionChange(new Selection(item));
                e.preventDefault();
                e.stopPropagation();
            }
        };

        return <HierarchyPills collections={collections} onCollectionClick={onClick} />;
    }
}
