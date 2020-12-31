import classNames from 'classnames';
import { exocore } from 'exocore';
import { memoize } from 'lodash';
import * as React from 'react';
import EmailFlows from '../../../logic/emails-logic';
import { exomind } from '../../../protos';
import { EntityTrait, EntityTraits } from '../../../store/entities';
import DateUtil from '../../../utils/date-util';
import DragAndDrop from '../../interaction/drag-and-drop/drag-and-drop';
import EditableText from '../../interaction/editable-text/editable-text';
import { EntityActions } from './entity-action';
import './entity.less';

interface IProps {
    entity: exocore.store.IEntity;
    parentEntity?: exocore.store.IEntity;

    selected?: boolean;
    onClick?: (e: MouseEvent) => void;
    actionsForEntity?: (entity: EntityTraits) => EntityActions;

    draggable?: boolean;
    droppable?: boolean;
    onDropOut?: (object: exocore.store.IEntity, effect: string, parentObject: exocore.store.IEntity) => void;
    onDropIn?: (object: exocore.store.IEntity, effect: string, parentObject: exocore.store.IEntity) => void;

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactFragment;
}

interface IState {
    removed: boolean;
    selected: boolean;
    hovered: boolean;
    beingDragged: boolean;
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
                actionsComponent = this.renderActions(entityTraits, actions);
            }
        }

        if (!actions) {
            actions = new EntityActions();
        }

        return (
            <li className={classes}
                onClick={this.handleItemClick.bind(this)}
                onMouseOver={this.handleItemMouseOver.bind(this)}
                onMouseLeave={this.handleItemMouseLeave.bind(this)}>

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

    private renderActions(entityTraits: EntityTraits, actions: EntityActions): React.ReactNode {
        const actionsComponents = actions.buttons.map((action) => {
            const classes = classNames({
                'action-icon': true,
                fa: true,
                ['fa-' + action.icon]: true
            });
            const cb = (e: MouseEvent) => {
                const result = action.trigger(e);
                e.stopPropagation();

                if (result == 'remove') {
                    this.removeItem();
                }
            };
            return (
                <li key={action.icon} onClick={cb.bind(this)}>
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
            emailThread: this.renderEmailThreadElement.bind(this, entityTraits),
            draftEmail: this.renderDraftEmailElement.bind(this),
            email: this.renderEmailElement.bind(this),
            collection: this.renderCollectionElement.bind(this),
            task: this.renderTaskElement.bind(this, actions),
            note: this.renderNoteElement.bind(this),
            link: this.renderLinkElement.bind(this),
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
                </div>

                <div className="clearfix" />
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
                </div>

                <div className="clearfix" />
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
                </div>

                <div className="clearfix" />
            </div>
        );
    }

    private renderCollectionElement(entityTrait: EntityTrait<exomind.base.ICollection>): React.ReactNode {
        const collection = entityTrait.message;

        return (
            <div className="item-container with-picture collection">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1"><span className="name">{collection.name}</span></div>
                </div>

                <div className="clearfix" />
            </div>
        );
    }

    private renderTaskElement(actions: EntityActions, entityTrait: EntityTrait<exomind.base.ITask>): React.ReactNode {
        const task = entityTrait.message;

        return (
            <div className="task item-container with-picture">
                {this.renderEntityImage(entityTrait)}
                <div className="date">{this.entityDate(entityTrait)}</div>
                <div className="content">
                    <div className="title1">
                        <EditableText
                            text={task.title}
                            initEdit={actions.inlineEdit}
                            onChange={this.handleTaskChange.bind(this, entityTrait, actions)}
                        />
                    </div>
                </div>

                <div className="clearfix" />
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
                </div>

                <div className="clearfix" />
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
                    <div className="title1">{entityTrait.displayName}</div>
                    <div className="text">{link.url}</div>
                </div>

                <div className="clearfix" />
            </div>
        );
    }

    private renderDefaultElement(entityTrait: EntityTrait<unknown>): React.ReactNode {
        return (
            <div className="default item-container with-picture">
                <div className="date">{this.entityDate(entityTrait)}</div>
                {this.renderEntityImage(entityTrait)}
                <div className="content">
                    <div className="title1">Unknown entity {entityTrait.et.id}</div>
                </div>

                <div className="clearfix" />
            </div>
        );
    }

    private renderEntityImage(entityTrait: EntityTrait<unknown>): React.ReactNode {
        const bubbleClasses = classNames({
            bubble: true,
            ['object-color-' + entityTrait.constants.color]: true
        });
        const iconClasses = classNames({
            'fa': true,
            'bubble-icon': true,
            ['fa-' + entityTrait.constants.icon]: true
        });

        return <div className="picture">
            <div className={bubbleClasses}>
                <div className={iconClasses} />
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

    private handleItemClick(e: MouseEvent): void {
        if (this.props.onClick) {
            this.props.onClick(e);
        }
    }

    private handleItemMouseOver(): void {
        this.setState({
            hovered: true
        });
    }

    private handleItemMouseLeave(): void {
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
