import React from 'react';
import EmailsLogic from '../../../logic/emails-logic';
import {EntityTraits, EntityTrait} from "../../../store/entities";
import {exomind} from '../../../protos';
import './email-attachments.less';

interface IProps {
    entity: EntityTraits;
    email: EntityTrait<exomind.base.IEmail>;
}

export class EmailAttachments extends React.Component<IProps> {

    render(): React.ReactNode {
        const attachments = this.props.email.message.attachments;
        if (attachments.length > 0) {
            return (
                <ul className="email-attachments">
                    {attachments.map(attach => this.renderAttachment(attach))}
                </ul>
            );
        } else {
            return null;
        }
    }

    private renderAttachment(attach: exomind.base.IEmailAttachment): React.ReactNode {
        return (
            <li key={attach.key}>
                <a href={EmailsLogic.attachmentUrl(this.props.entity, this.props.email, attach)} target="_blank" rel="noreferrer">
                <span className="icon"/>
                <span className="text">{attach.name ?? 'Unnamed'}</span></a>
            </li>
        );
    }
}

