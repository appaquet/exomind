import classNames from 'classnames';
import { fromProtoTimestamp, MutationBuilder, Exocore } from 'exocore';
import { exomind } from '../../../protos';
import _ from 'lodash';
import React from 'react';
import EmailUtil from '../../../utils/emails';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { SelectedItem, Selection } from '../entity-list/selection';
import { EmailAttachments } from './email-attachments';
import { ContainerState } from '../container-state';
import { observer } from 'mobx-react';
import './email-thread.less';

interface IProps {
    entity?: EntityTraits;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

interface IState {
    collapsed: boolean;
    emailStates: EmailState[];
    controlledEmailId?: string;
    hoveredEmailId?: string;
}

interface EmailState {
    index: number;
    trait: EntityTrait<exomind.base.v1.Email | exomind.base.v1.DraftEmail>;
    isOpen: boolean;
    cleanedCurrent?: string;
    original?: string;
}

@observer
export default class EmailThread extends React.Component<IProps, IState> {
    private mounted = false;
    private draftTrait?: EntityTrait<exomind.base.v1.DraftEmail>;
    private threadElement: React.RefObject<HTMLUListElement> = React.createRef();

    constructor(props: IProps) {
        super(props);

        this.draftTrait = props.entity.traitOfType(exomind.base.v1.DraftEmail);

        const unreadFlags: { [id: string]: unknown } = _.chain(props.entity.traitsOfType<exomind.base.v1.Unread>(exomind.base.v1.Unread))
            .groupBy((flag) => {
                return flag.message.entity?.traitId;
            })
            .value();

        let count = 0;
        const emailStates = _.chain(props.entity.traitsOfType<exomind.base.v1.Email>(exomind.base.v1.Email))
            .sortBy((a) => {
                return fromProtoTimestamp(a.message.receivedDate).getTime();
            })
            .map((trait) => {
                const unread = trait.id in unreadFlags;
                return { index: count++, trait: trait, isOpen: unread } as EmailState;
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

    componentDidMount(): void {
        this.mounted = true;

        if ((this.props.containerState?.active ?? false) && this.threadElement.current) {
            this.threadElement.current.focus();
        }
    }

    componentWillUnmount(): void {
        this.mounted = false;
    }

    componentDidUpdate(): void {
        if ((this.props.containerState?.active ?? false) && this.threadElement.current) {
            this.threadElement.current.focus();
        }
    }

    render(): React.ReactNode {
        const classes = classNames({
            'email-thread': true,
            'entity-component': true,
            'active': this.props.containerState?.active ?? false,
        });

        const emails = this.renderEmails();
        return (
            <div className={classes} onMouseLeave={this.handleThreadMouseLeave}>
                <ul className="thread" tabIndex={0} ref={this.threadElement}>
                    {emails}
                </ul>
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
                    }) as React.ReactNode[];
                }
            })
            .flatten()
            .value();
    }

    private renderEmail(emailState: EmailState, email: EntityTrait<exomind.base.v1.IEmail>): React.ReactNode {
        const open = emailState.isOpen;
        const hovered = this.state.hoveredEmailId === email.id && (this.props.containerState?.active ?? true);
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
                onMouseEnter={() => this.handleEmailMouseEnter(emailState, email)}
                onMouseLeave={this.handleEmailMouseLeave}>

                <div className="preview-header" onClick={() => this.handleEmailClick(emailState, email)}>
                    <span className="from">{EmailUtil.formatContact(email.message.from)}</span>
                    {snippetOrTo}
                    <span
                        className="time">{EmailUtil.formatDate(fromProtoTimestamp(email.message.receivedDate))}
                    </span>
                    <span className="header-controls" onClick={() => this.handleOpenEmailClick(email)}>
                        <i className="icon" />
                    </span>
                </div>

                {emailBody}
            </li>
        );
    }

    private renderSnippet(email: EntityTrait<exomind.base.v1.IEmail>): React.ReactNode {
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
                <div className="preview-header" onClick={this.handleCollapsedClick}>
                    <div className="line">&nbsp;</div>
                    <div className="line">&nbsp;</div>
                    <div className="count">{count}</div>
                </div>
            </li>
        );
    }

    private renderDraftEmail(state: EmailState, draft: EntityTrait<exomind.base.v1.IDraftEmail>): React.ReactNode {
        const classes = classNames({
            draft: true
        });

        return <li key={draft.id} className={classes}>
            <div className="preview-header" onClick={() => this.handleDraftClick(draft)}>
                <span className="snippet">Draft reply</span>
                <span className="time">{EmailUtil.formatDate(draft.modificationDate ?? new Date())}</span>
                <span className="header-controls" onClick={() => this.handleDraftClick(draft)}><i
                    className="icon" /></span>
            </div>
        </li>;
    }

    private renderToContacts(email: EntityTrait<exomind.base.v1.IEmail>): React.ReactNode {
        const to = _(email.message.to).map(contact => EmailUtil.formatContact(contact)).value();
        const cc = _(email.message.cc).map(contact => EmailUtil.formatContact(contact)).value();
        const bcc = _(email.message.bcc).map(contact => EmailUtil.formatContact(contact)).value();
        const text = to.concat(cc).concat(bcc).join(', ');

        return <span className="to">to {text}</span>;
    }

    private renderEmailBody(emailState: EmailState, email: EntityTrait<exomind.base.v1.IEmail>): React.ReactNode {
        const htmlPart = EmailUtil.extractHtmlPart(email.message.parts);
        const textPart = EmailUtil.extractTextPart(email.message.parts);

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
                <div className="more" onClick={() => this.handleOpenEmailClick(email)}><span className="icon" />
                </div> : null;

        } else if (textPart) {
            const body = textPart?.body ?? '';
            markup = { __html: EmailUtil.plainTextToHtml(body) };
        } else {
            markup = { __html: "<b style='color:red'>Couldn't find part to render</b>" };
        }

        return (
            <div className="object-body">
                <div className="email-body" dangerouslySetInnerHTML={markup} />
                {more}
                <EmailAttachments entity={this.props.entity} email={email} />
            </div>
        );
    }

    private handleCollapsedClick = (): void => {
        this.setState({
            collapsed: false
        });
    };

    private handleOpenEmailClick(email: EntityTrait<exomind.base.v1.IEmail>): void {
        if (this.props.onSelectionChange) {
            const item = SelectedItem.fromEntityTraitId(this.props.entity.entity.id, email.id);
            this.props.onSelectionChange(this.props.selection.withItem(item));
        }
    }

    private handleThreadMouseLeave = (): void => {
        this.setState({
            controlledEmailId: null
        });
    };

    private handleEmailMouseEnter(emailState: EmailState, email: EntityTrait<exomind.base.v1.IEmail>): void {
        this.setState({
            hoveredEmailId: email.id,
            controlledEmailId: emailState.isOpen ? email.id : this.state.controlledEmailId // show control if we mouse over email and it's open
        });
    }

    private handleEmailMouseLeave = (): void => {
        this.setState({
            hoveredEmailId: null,
        });
    };

    private handleEmailClick(emailState: EmailState, email: EntityTrait<exomind.base.v1.IEmail>): void {
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

    private handleDraftClick = (draft: EntityTrait<exomind.base.v1.IDraftEmail>): void => {
        if (this.props.onSelectionChange) {
            const item = SelectedItem.fromEntityTraitId(this.props.entity.entity.id, draft.id);
            this.props.onSelectionChange(this.props.selection.withItem(item));
        }
    };

    private trackMarkAsRead(): void {
        const entity = this.props.entity;
        setTimeout(() => {
            if (!this.mounted || entity.id != this.props.entity.id) {
                return;
            }

            const unreadFlags = entity.traitsOfType(exomind.base.v1.Unread);
            if (unreadFlags.length == 0) {
                return;
            }

            let mb = MutationBuilder.updateEntity(entity.id);
            for (const flag of unreadFlags) {
                mb = mb.deleteTrait(flag.id);
            }

            Exocore.store.mutate(mb.build());
        }, 3000);
    }
}

