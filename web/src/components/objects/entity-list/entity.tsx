import classNames from 'classnames';
import * as React from 'react';
import EmailFlows from '../../../utils/emails';
import { exomind } from '../../../protos';
import { EntityParent, getEntityParentWeight, Parents, PINNED_WEIGHT } from '../../../stores/collections';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import DateUtil from '../../../utils/dates';
import DragAndDrop, { DragData, DropPosition } from '../../interaction/drag-and-drop/drag-and-drop';
import EditableText from '../../interaction/editable-text/editable-text';
import EntityIcon from '../entity-icon';
import { HierarchyPills } from '../hierarchy-pills/hierarchy-pills';
import { SelectedItem, Selection } from "./selection";
import { ListEntityAction, ListEntityActions } from './actions';
import { observer } from 'mobx-react';
import { IStores, StoresContext } from '../../../stores/stores';
import './entity.less';

export interface IProps {
    id: string;
    entity: EntityTraits;
    parentEntity?: EntityTraits;

    active?: boolean;

    selected?: boolean;
    onSelectionChange?: (sel: Selection) => void;
    onClick?: (e: React.MouseEvent) => void;
    actionsForEntity?: (entity: EntityTraits) => ListEntityActions;

    onMouseLeave?: (e: React.MouseEvent) => void;
    onMouseOver?: (e: React.MouseEvent) => void;

    draggable?: boolean;
    droppable?: boolean;
    onDropOut?: (data: DragData) => void;
    onDropIn?: (data: DragData) => void;

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactNode;
}

interface IState {
    removed: boolean;
    selected: boolean;
    beingDragged: boolean;
    parents?: Parents,
}

export class Entity extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    constructor(props: IProps) {
        super(props);

        this.state = {
            removed: false,
            selected: props.selected ?? false,
            beingDragged: false,
        };
    }

    render(): React.ReactNode {
        const active = this.props.active === true;
        const classes = classNames({
            item: true,
            done: this.state.removed,
            selected: (this.props.selected === true),
            hover: active,
        });

        let actionsComponent = null;
        let actions: ListEntityActions | null = null;
        if (this.props.actionsForEntity) {
            actions = this.props.actionsForEntity(this.props.entity);
            if (active && !actions.isEmpty) {
                actionsComponent = this.renderActions(actions);
            }
        }
        if (!actions) {
            actions = new ListEntityActions();
        }

        let dropPositions: DropPosition[];
        const priorityTrait = this.props.entity.priorityTrait;
        if (priorityTrait && (priorityTrait.constants.collectionLike ?? false)) {
            // collections supports dropping inside, so we include 'middle'
            dropPositions = ['before', 'into', 'after'];
        }

        return (
            <li id={this.props.id}
                className={classes}
                onClick={this.handleItemClick}
                onMouseOver={this.props.onMouseOver}
                onMouseLeave={this.props.onMouseLeave}
                onContextMenu={actions ? (e) => this.handleContextMenu(e, actions) : undefined}
            >
                <div className="swipe-container">
                    <DragAndDrop
                        object={this.props.entity}
                        parentObject={this.props.parentEntity}
                        draggable={this.props.draggable}
                        droppable={this.props.droppable}
                        dropPositions={dropPositions}
                        onDropIn={this.props.onDropIn}
                        onDropOut={this.props.onDropOut}>

                        {actionsComponent}

                        {this.renderElement(this.props.entity, actions)}
                    </DragAndDrop>
                </div>
            </li>
        );
    }

    private renderActions(actions: ListEntityActions): React.ReactNode {
        let limitedButtons = actions.buttons;
        if (actions.buttons.length > 3) {
            limitedButtons = actions.buttons.slice(0, 3);

            limitedButtons.push(new ListEntityAction('More', 'ellipsis-v', async (action, event) => {
                this.context.session.showMenu({
                    items: actions.toMenuItems(),
                }, event.currentTarget as HTMLElement);
            }));
        }

        const actionsComponents = limitedButtons.map((action) => {
            const classes = classNames({
                'action-icon': true,
                fa: true,
                ['fa-' + action.icon]: true
            });
            const cb = async (e: React.MouseEvent) => {
                e.stopPropagation();

                const result = await action.trigger(e);
                if (result == 'remove') {
                    this.removeItem();
                    this.props.onSelectionChange?.(new Selection());
                }
            };
            return (
                <li key={action.icon} onClick={cb}>
                    <span className={classes} />
                </li>
            );
        });

        return (
            <div className="item-actions-container">
                <div className="item-actions">
                    <ul>{actionsComponents}</ul>
                </div>
            </div>
        );
    }

    private renderElement(entityTraits: EntityTraits, actions: ListEntityActions): React.ReactNode {
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
        }) as React.ReactNode;
    }

    private renderEmailThreadElement(entityTraits: EntityTraits, entityTrait: EntityTrait<exomind.base.v1.IEmailThread>): React.ReactNode {
        const thread = entityTrait.message;
        const emails = entityTraits.traitsOfType<exomind.base.v1.IEmail>(exomind.base.v1.Email);
        const unreadFlags = entityTraits.traitsOfType<exomind.base.v1.IUnread>(exomind.base.v1.Unread);

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
            snippetMarkup = <div className="text">{thread.snippet}</div>;
        }

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'email-thread': true,
            'with-indicators': !!indicators,
            unread: unreadFlags.length > 0,
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
                {indicators}
            </div>
        );
    }

    private renderDraftEmailElement(entityTrait: EntityTrait<exomind.base.v1.IDraftEmail>): React.ReactNode {
        const draft = entityTrait.message;

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
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
                {indicators}
            </div>
        );
    }

    private renderEmailElement(entityTrait: EntityTrait<exomind.base.v1.IEmail>): React.ReactNode {
        const email = entityTrait.message;

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
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
                {indicators}
            </div>
        );
    }

    private renderCollectionElement(entityTrait: EntityTrait<exomind.base.v1.ICollection>): React.ReactNode {
        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
            collection: true,
        });

        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{entityTrait.displayName}</span></div>
                    {entityTrait.message.description &&
                        <div className="text">{entityTrait.message.description}</div>
                    }
                    {this.renderParents(entityTrait.et)}
                </div>
                {indicators}
            </div>
        );
    }

    private renderTaskElement(actions: ListEntityActions, entityTrait: EntityTrait<exomind.base.v1.ITask>): React.ReactNode {
        const task = entityTrait.message;
        const onTitleChange = (newTitle: string) => {
            this.handleTaskChange(entityTrait, actions, newTitle);
        };

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
            'task': true,
        });

        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1">
                        <EditableText
                            text={task.title}
                            initializeEditing={!!actions.inlineAction}
                            onChange={onTitleChange}
                        />
                    </div>
                    {this.renderParents(entityTrait.et)}
                </div>
                {indicators}
            </div>
        );
    }

    private renderNoteElement(entityTrait: EntityTrait<exomind.base.v1.INote>): React.ReactNode {
        const note = entityTrait.message;

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
            'note': true,
        });

        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{note.title}</span></div>
                    {this.renderParents(entityTrait.et)}
                </div>
                {indicators}
            </div>
        );
    }

    private renderLinkElement(entityTrait: EntityTrait<exomind.base.v1.ILink>): React.ReactNode {
        const link = entityTrait.message;

        const indicators = this.renderIndicators();
        const classes = classNames({
            'item-container': true,
            'with-picture': true,
            'with-indicators': !!indicators,
            'link': true,
        });

        return (
            <div className={classes}>
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    {entityTrait.displayName && <div className="title1">{entityTrait.displayName}</div>}
                    <div className="text">{link.url}</div>
                    {this.renderParents(entityTrait.et)}
                </div>
                {indicators}
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

    private renderIndicators(): React.ReactNode {
        const isPinned = this.isPinned;
        const isSnoozed = this.isSnoozed;
        if (!isPinned && !isSnoozed) {
            return null;
        }

        return (
            <div className="indicators">
                {isPinned ? <span className="pinned" /> : null}
                {isSnoozed ? <span className="snoozed" /> : null}
            </div>
        );
    }

    private entityDate(entityTrait: EntityTrait<unknown>): React.ReactNode {
        if (this.props.renderEntityDate) {
            return this.props.renderEntityDate(entityTrait);
        } else {
            return DateUtil.toShortFormat(entityTrait.modificationDate ?? entityTrait.creationDate ?? new Date());
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
    };

    private handleContextMenu(e: React.MouseEvent, actions: ListEntityActions): void {
        this.context.session.showMenu({
            items: actions.toMenuItems(),
            mouseEvent: e,
        });
        e.preventDefault();
        e.stopPropagation();
    }

    private handleTaskChange(task: EntityTrait<exomind.base.v1.ITask>, actions: ListEntityActions, newTitle: string): void {
        task.rename(newTitle);

        if (actions.inlineAction) {
            actions.inlineAction.trigger();
        }
    }

    private get isPinned(): boolean {
        if (!this.props.parentEntity) {
            return false;
        }

        const weight = getEntityParentWeight(this.props.entity, this.props.parentEntity.id);
        return weight >= PINNED_WEIGHT;
    }

    private get isSnoozed(): boolean {
        const snoozed = this.props.entity.traitOfType<exomind.base.v1.ISnoozed>(exomind.base.v1.Snoozed);
        return !!snoozed;
    }
}

interface EntityParentsProps {
    entity: EntityTraits;
    parentEntity?: EntityTraits;
    onSelectionChange?: (sel: Selection) => void;
}

@observer
class EntityParents extends React.Component<EntityParentsProps> {
    static contextType = StoresContext;
    declare context: IStores;

    render(): React.ReactNode {
        const parents = this.context.collections.getEntityParents(this.props.entity);
        if (!parents) {
            return <span className="loading"></span>;
        }

        const collections = parents.get().filter((col) => {
            return col.entityId != this.props.parentEntity?.id;
        });

        const onClick = (e: React.MouseEvent, col: EntityParent) => {
            if (this.props.onSelectionChange) {
                const item = SelectedItem.fromEntityId(col.entityId);
                this.props.onSelectionChange(new Selection(item));
                e.stopPropagation();
            }
        };

        return <HierarchyPills collections={collections} onCollectionClick={onClick} />;
    }
}
