import classNames from 'classnames';
import { fromProtoTimestamp, MutationBuilder, Exocore } from 'exocore';
import { exomind } from '../../../protos';
import _ from 'lodash';
import React from 'react';
import EmailUtil from '../../../utils/emails';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { SelectedItem, Selection } from '../entity-list/selection';
import { EmailAttachments } from './email-attachments';
import './email-thread.less';
import { ContainerController } from '../container-controller';
import { runInAction } from 'mobx';


interface IProps {
    entity?: EntityTraits;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerController?: ContainerController;
}

interface IState {
    collapsed: boolean;
    emailStates: EmailState[];
    controlledEmailId?: string;
    hoveredEmailId?: string;
}

interface EmailState {
    index: number;
    trait: EntityTrait<exomind.base.Email | exomind.base.DraftEmail>;
    isOpen: boolean;
    cleanedCurrent?: string;
    original?: string;
}

export default class EmailThread extends React.Component<IProps, IState> {
    threadTrait: EntityTrait<exomind.base.EmailThread>;
    draftTrait?: EntityTrait<exomind.base.DraftEmail>;

    constructor(props: IProps) {
        super(props);

        this.threadTrait = props.entity.traitOfType(exomind.base.EmailThread);
        this.draftTrait = props.entity.traitOfType(exomind.base.DraftEmail);

        let count = 0;
        const emailStates = _.chain(props.entity.traitsOfType<exomind.base.Email>(exomind.base.Email))
            .map((trait) => {
                return { index: count++, trait: trait, isOpen: !trait.message.read } as EmailState;
            })
            .sortBy((a) => {
                return a.trait.modificationDate ?? a.trait.creationDate;
            })
            .value();

        if (this.draftTrait) {
            emailStates.push({ index: count++, trait: this.draftTrait, isOpen: true });
        }

        const last = _.last(emailStates);
        if (last) {
            last.isOpen = true;
        }

        this.trackMarkAsRead();

        this.state = {
            collapsed: true,
            emailStates: emailStates,
            controlledEmailId: null,
        };
    }

    render(): React.ReactNode {
        const emails = this.renderEmails();
        const emailControls = (this.state.controlledEmailId) ? this.renderEmailControls() : null;
        return (
            <div className="entity-component email-thread" onMouseLeave={this.handleThreadMouseLeave.bind(this)}>
                <ul className="thread">
                    {emails}
                </ul>
                {emailControls}
            </div>
        );
    }

    private renderEmails(): React.ReactNode {
        let emailsStates: (EmailState | string)[] = this.state.emailStates;
        const count = this.state.emailStates.length;
        if (this.state.collapsed && count > 4) {
            emailsStates = [emailsStates[0], 'collapsed', emailsStates[count - 2], emailsStates[count - 1]];
        }

        return _(emailsStates)
            .map((state) => {
                if (typeof state == 'string') {
                    return [this.renderCollapsed(count)];
                } else {
                    return state.trait.match({
                        email: email => {
                            return [this.renderEmail(state, email)];
                        },
                        draftEmail: draft => {
                            return [this.renderDraftEmail(state, draft)];
                        },
                        default: () => {
                            return [(
                                <li key={state.trait.id} className="email opened">
                                    {state.trait.constants.key}
                                </li>
                            )];
                        },
                    });
                }
            })
            .flatten()
            .value();
    }

    private renderEmail(emailState: EmailState, email: EntityTrait<exomind.base.IEmail>): React.ReactNode {
        const open = emailState.isOpen;
        const hovered = this.state.hoveredEmailId === email.id;
        const classes = classNames({
            email: true,
            opened: open,
            closed: !open,
            hovered: hovered
        });

        const snippetOrTo = (!open) ? this.renderSnippet(email) : this.renderToContacts(email);
        const emailBody = (open) ? this.renderEmailBody(emailState, email) : null;

        return (
            <li key={email.id}
                className={classes}
                onMouseEnter={this.handleEmailMouseEnter.bind(this, emailState, email)}
                onMouseLeave={this.handleEmailMouseLeave.bind(this)}>

                <div className="preview-header" onClick={this.handleEmailClick.bind(this, emailState, email)}>
                    <span className="from">{EmailUtil.formatContact(email.message.from)}</span>
                    {snippetOrTo}
                    <span
                        className="time">{EmailUtil.formatDate(fromProtoTimestamp(email.message.receivedDate))}
                    </span>
                    <span className="header-controls" onClick={this.handleOpenEmailClick.bind(this, emailState, email)}>
                        <i className="icon" />
                    </span>
                </div>

                {emailBody}
            </li>
        );
    }

    private renderSnippet(email: EntityTrait<exomind.base.IEmail>): React.ReactNode {
        const snippet = email.message.snippet ?? email.message.subject ?? '';
        return <span className="snippet">{snippet}</span>;
    }

    private renderCollapsed(count: number): React.ReactNode {
        const classes = classNames({
            email: true,
            collapsed: true,
            open: false,
            closed: true,
            hovered: false
        });

        return (
            <li key="collapsed" className={classes}>
                <div className="preview-header" onClick={this.handleCollapsedClick.bind(this)}>
                    <div className="line">&nbsp;</div>
                    <div className="line">&nbsp;</div>
                    <div className="count">{count}</div>
                </div>
            </li>
        );
    }

    private renderDraftEmail(state: EmailState, draft: EntityTrait<exomind.base.IDraftEmail>): React.ReactNode {
        const classes = classNames({
            draft: true
        });

        return <li key={draft.id} className={classes}>
            <div className="preview-header" onClick={this.handleDraftClick.bind(this, draft)}>
                <span className="snippet">Draft reply</span>
                <span className="time">{EmailUtil.formatDate(draft.modificationDate ?? new Date())}</span>
                <span className="header-controls" onClick={this.handleDraftClick.bind(this, draft)}><i
                    className="icon" /></span>
            </div>
        </li>;
    }

    private renderToContacts(email: EntityTrait<exomind.base.IEmail>): React.ReactNode {
        const to = _(email.message.to).map(contact => EmailUtil.formatContact(contact)).value();
        const cc = _(email.message.cc).map(contact => EmailUtil.formatContact(contact)).value();
        const bcc = _(email.message.bcc).map(contact => EmailUtil.formatContact(contact)).value();
        const text = to.concat(cc).concat(bcc).join(', ');

        return <span className="to">to {text}</span>;
    }

    private renderEmailBody(emailState: EmailState, email: EntityTrait<exomind.base.IEmail>): React.ReactNode {
        const htmlPart = EmailUtil.extractHtmlPart(email.message.parts);

        let markup = { __html: '' };
        let more = null;
        if (htmlPart) {
            // cache cleaned html into email state
            if (!emailState.cleanedCurrent) {
                const body = htmlPart.body;

                // if it's not first email in thread, we try to split new content from old content
                let current, original;
                if (emailState.index > 0) {
                    [current, original] = EmailUtil.splitOriginalThreadHtml(body);
                } else {
                    [current, original] = [body, ''];
                }

                const currentWithAttachment = EmailUtil.injectInlineImages(this.props.entity, email, current);
                emailState.cleanedCurrent = EmailUtil.sanitizeHtml(currentWithAttachment);
                emailState.original = original;
            }

            markup = { __html: emailState.cleanedCurrent };
            more = (!_.isEmpty(emailState.original)) ?
                <div className="more" onClick={this.handleOpenEmailClick.bind(this, email)}><span className="icon" />
                </div> : null;

        } else if (!_.isEmpty(email.message.parts)) {
            const body = _.first(email.message.parts)?.body ?? '';
            markup = { __html: EmailUtil.plainTextToHtml(body) };
        }

        return (
            <div className="object-body">
                <div className="email-body" dangerouslySetInnerHTML={markup} />
                {more}
                <EmailAttachments entity={this.props.entity} email={email} />
            </div>
        );
    }

    private handleCollapsedClick(): void {
        this.setState({
            collapsed: false
        });
    }

    private handleOpenEmailClick(emailState: EmailState, email: EntityTrait<exomind.base.IEmail>): void {
        if (this.props.onSelectionChange) {
            const item = SelectedItem.fromEntityTraitId(this.props.entity.entity.id, email.id);
            this.props.onSelectionChange(this.props.selection.withItem(item));
        }
    }

    private handleThreadMouseLeave(): void {
        this.setState({
            controlledEmailId: null
        });
    }

    private handleEmailMouseEnter(emailState: EmailState, email: EntityTrait<exomind.base.IEmail>): void {
        this.setState({
            hoveredEmailId: email.id,
            controlledEmailId: emailState.isOpen ? email.id : this.state.controlledEmailId // show control if we mouse over email and it's open
        });
    }

    private handleEmailMouseLeave(): void {
        this.setState({
            hoveredEmailId: null,
        });
    }

    private handleEmailClick(emailState: EmailState, email: EntityTrait<exomind.base.IEmail>): void {
        emailState.isOpen = !emailState.isOpen;

        // if the email for which we show controls is the one we close, we need to remove controls
        let emailWithControlsShown = this.state.controlledEmailId;
        if (!emailState.isOpen && this.state.controlledEmailId === email.id) {
            emailWithControlsShown = null;
        }

        // toggle state
        this.setState({
            controlledEmailId: emailWithControlsShown
        });
    }

    private handleDraftClick(draft: EntityTrait<exomind.base.IDraftEmail>): void {
        if (this.props.onSelectionChange) {
            const item = SelectedItem.fromEntityTraitId(this.props.entity.entity.id, draft.id);
            this.props.onSelectionChange(this.props.selection.withItem(item));
        }
    }

    private renderEmailControls(): React.ReactNode {
        let doneAction = null;
        const inboxChild = this.props.entity
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
            .find((trt) => trt.message.collection.entityId == 'inbox');
        if (inboxChild) {
            doneAction = <>
                <li onClick={this.handleDoneEmail.bind(this, inboxChild)}><i className="done" /></li>
            </>;
        }

        return (
            <div className="list-actions">
                <ul>
                    {doneAction}
                    <li onClick={this.handleReplyAllEmail.bind(this)}><i className="reply-all" /></li>
                    <li onClick={this.handleReplyEmail.bind(this)}><i className="reply" /></li>
                    <li onClick={this.handleForwardEmail.bind(this)}><i className="forward" /></li>
                </ul>
            </div>
        );
    }

    private handleDoneEmail(child: EntityTrait<exomind.base.CollectionChild>): void {
        const mutation = MutationBuilder
            .updateEntity(this.props.entity.id)
            .deleteTrait(child.id)
            .build();
        Exocore.store.mutate(mutation);

        if (this.props.containerController) {
            runInAction(() => {
                this.props.containerController.closed = true;
            });
        }
    }

    private handleReplyEmail(): void {
        // TODO: Reply
        // EmailsLogicXYZ.createReplyEmail(this.props.entity, email).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         let entityTrait = new EntityTrait(this.props.entity, 'exomind.draft_email');
        //         this.props.onSelectionChange([entityTrait]);
        //     }
        // });
    }

    private handleReplyAllEmail(): void {
        // TODO: Reply all
        // EmailsLogicXYZ.createReplyAllEmail(this.props.entity, email).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         let entityTrait = new EntityTrait(this.props.entity.id, 'exomind.draft_email');
        //         this.props.onSelectionChange([entityTrait]);
        //     }
        // });
    }

    private handleForwardEmail(): void {
        // TODO: Forward
        // EmailsLogicXYZ.createForwardEmail(this.props.entity, email).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         let entityTrait = new EntityTrait(this.props.entity.id, 'exomind.draft_email');
        //         this.props.onSelectionChange([entityTrait]);
        //     }
        // });
    }

    private trackMarkAsRead(): void {
        // TODO: Mark as read
        // let entity = this.props.entity;
        // setTimeout(t => {
        //     if (this.mounted && entity.id == this.props.entity.id && this.markRead !== entity.id) {
        //
        //         let emails = this.emailOrDraftTraits();
        //         let unreadTraits = _(emails)
        //             .map((email) => {
        //                 let isUnread = (email instanceof Exomind.Email) ? email.unread.getOrElse(false) : false;
        //                 if (isUnread) {
        //                     let emailBuilder = new Exomind.Email();
        //                     emailBuilder.id = email.id;
        //                     emailBuilder.unread = false;
        //                     return [emailBuilder];
        //                 } else {
        //                     return [];
        //                 }
        //             })
        //             .flatten()
        //             .value();
        //
        //         if (!_.isEmpty(unreadTraits)) {
        //             ExomindDSL.on(entity).mutate.update(unreadTraits).execute();
        //         }
        //
        //         this.markRead = entity.id;
        //     }
        // }, 3000);
    }

}

