import classNames from 'classnames';
import React from 'react';
import EmailsLogic from '../../../logic/emails-logic';
import { exomind } from '../../../protos';
import { EntityTrait, EntityTraits } from '../../../store/entities';
import { Selection } from '../entity-list/selection';
import { EmailAttachments } from './email-attachments';
import './email.less';

interface IProps {
    entity: EntityTraits;
    emailTrait: EntityTrait<exomind.base.IEmail>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
}

interface IState {
    hovered: boolean;
}

export default class Email extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);

        this.state = {
            hovered: false
        };
    }

    render(): React.ReactNode {
        const email = this.props.emailTrait;
        const emailControls = (this.state.hovered) ? this.renderEmailControls() : null;

        return (
            <div className="entity-component email" onMouseOver={this.handleMouseOver.bind(this)}
                onMouseLeave={this.handleMouseLeave.bind(this)}>
                <div className="object-summary">
                    <div className="from field">
                        <span className="field-label">From</span>
                        <span className="pill">{EmailsLogic.formatContact(email.message.from)}</span>
                    </div>
                    {this.renderContactField('to', 'To', email.message.to)}
                    {this.renderContactField('cc', 'CC', email.message.cc)}
                    {this.renderContactField('bcc', 'BCC', email.message.bcc)}
                </div>

                <div className="object-body">
                    {this.renderBody()}

                    <EmailAttachments entity={this.props.entity} email={email} />
                </div>

                {emailControls}
            </div>
        );
    }

    private handleMouseOver(): void {
        if (!this.state.hovered) {
            this.setState({
                hovered: true
            });
        }
    }

    private handleMouseLeave(): void {
        this.setState({
            hovered: false
        });
    }

    private renderContactField(key: string, label: string, contacts: exomind.base.IContact[]): React.ReactNode {
        if (contacts.length > 0) {
            const classes = classNames({
                field: true,
                [key]: true
            });
            const pills = contacts.map(contact => {
                return <span className="pill" key={contact.email}>{EmailsLogic.formatContact(contact)}</span>
            });
            return <div className={classes}><span className="field-label">{label}</span> {pills}</div>
        }
    }

    private renderBody(): React.ReactNode {
        const email = this.props.emailTrait;
        const htmlPart = EmailsLogic.extractHtmlPart(email.message.parts);
        if (htmlPart) {
            const bodyWithAttachment = EmailsLogic.injectInlineImages(this.props.entity, this.props.emailTrait, htmlPart.body);
            const cleaned = EmailsLogic.sanitizeHtml(bodyWithAttachment);
            const markup = { __html: cleaned };
            return <div dangerouslySetInnerHTML={markup} />;

        } else if (email.message.parts.length > 0) {
            return <pre>{email.message.parts[0].body}</pre>;
        }
    }

    private renderEmailControls(): React.ReactNode {
        return <div className="object-actions">
            <ul>
                <li onClick={this.handleReplyAllEmail.bind(this)}><i className="reply-all" /></li>
                <li onClick={this.handleReplyEmail.bind(this)}><i className="reply" /></li>
                <li onClick={this.handleForwardEmail.bind(this)}><i className="forward" /></li>
            </ul>
        </div>
    }

    private handleReplyEmail(): void {
        // TODO:
        // EmailsLogic.createReplyEmail(this.props.entity, this.props.emailTrait).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         this.props.onSelectionChange([obj]);
        //     }
        // });
    }

    private handleReplyAllEmail(): void {
        // TODO:
        // EmailsLogic.createReplyAllEmail(this.props.entity, this.props.emailTrait).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         this.props.onSelectionChange([obj]);
        //     }
        // });
    }

    private handleForwardEmail(): void {
        // TODO:
        // EmailsLogic.createForwardEmail(this.props.entity, this.props.emailTrait).onProcessed((cmd, obj) => {
        //     if (obj) {
        //         this.props.onSelectionChange([obj]);
        //     }
        // });
    }
}
