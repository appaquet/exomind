import React from 'react';
import EmailUtil from '../../../utils/emails';
import {EntityTraits, EntityTrait} from "../../../utils/entities";
import {exomind} from '../../../protos';
import './email-attachments.less';

interface IProps {
    entity: EntityTraits;
    email: EntityTrait<exomind.base.v1.IEmail>;
}

export class EmailAttachments extends React.Component<IProps> {

    render(): React.ReactNode {
        const attachments = this.props.email.message.attachments;
        if (attachments.length > 0) {
            return (
                <ul className="email-attachments">
                    {attachments.map(this.renderAttachment)}
                </ul>
            );
        } else {
            return null;
        }
    }

    private renderAttachment = (attach: exomind.base.v1.IEmailAttachment, index: number): React.ReactNode => {
        return (
            <li key={`${attach.key}${index}`}>
                <a href={EmailUtil.attachmentUrl(this.props.entity, this.props.email, attach)} target="_blank" rel="noreferrer">
                <span className="icon"/>
                <span className="text">{attach.name ?? 'Unnamed'}</span></a>
            </li>
        );
    };
}

