
import classNames from 'classnames';
import { Exocore, MutationBuilder, QueryBuilder } from 'exocore';
import _ from 'lodash';
import React, { ChangeEvent } from 'react';
import EmailUtil from '../../../utils/emails';
import { exocore, exomind } from '../../../protos';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import HtmlEditor from '../../interaction/html-editor/html-editor';
import { ContainerState } from '../container-state';
import { Selection } from '../entity-list/selection';
import './draft-email.less';

interface IProps {
    entity: EntityTraits;
    draftTrait: EntityTrait<exomind.base.v1.IDraftEmail>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

type AccountsMap = { [entity_trait_id: string]: { entity: EntityTraits, account: EntityTrait<exomind.base.v1.IAccount> } };

interface IState {
    savedDraft: exomind.base.v1.IDraftEmail;
    currentDraft: exomind.base.v1.IDraftEmail;
    accounts?: AccountsMap;
    editor?: HtmlEditor;
}

export default class DraftEmail extends React.Component<IProps, IState> {
    private mounted = true;

    constructor(props: IProps) {
        super(props);

        this.fetchAccounts();

        this.state = {
            savedDraft: props.draftTrait.message,
            currentDraft: new exomind.base.v1.DraftEmail(props.draftTrait.message),
        };
    }

    componentWillUnmount(): void {
        this.saveObject();
        this.mounted = false;
    }

    render(): React.ReactNode {
        const subject = this.state.currentDraft.subject;

        return (
            <div className="entity-component draft-email">
                <div className="entity-details">
                    {this.renderFromAccount()}
                    {this.renderContactField('to', 'To')}
                    {this.renderContactField('cc', 'CC')}
                    {this.renderContactField('bcc', 'BCC')}

                    <div className="subject field">
                        <span className="field-label">Subject</span>
                        <span className="field-content">
                            <input type="text"
                                value={subject}
                                onChange={this.handleSubjectChange}
                                onBlur={this.saveObject}
                                placeholder="Type an email subject here" />
                        </span>
                    </div>
                </div>

                <div className="object-body">
                    {this.renderBody()}
                </div>
            </div>
        );
    }

    private async fetchAccounts(): Promise<void> {
        const results = await Exocore.store.query(QueryBuilder.withTrait(exomind.base.v1.Account).build());

        const accounts = results.entities
            .map((res) => {
                return new EntityTraits(res.entity);
            })
            .flatMap((entity) => {
                const accounts = entity.traitsOfType<exomind.base.v1.IAccount>(exomind.base.v1.Account);
                return accounts.map((account) => {
                    return { entity, account };
                });
            });

        const accountsMap: AccountsMap = {};
        for (const { entity, account } of accounts) {
            accountsMap[entity.id + account.id] = { entity, account };
        }

        this.setState({ accounts: accountsMap });
    }

    private handleSubjectChange = (e: ChangeEvent<HTMLInputElement>): void => {
        const draft = this.state.currentDraft;
        draft.subject = e.target.value;
        this.setState({ currentDraft: draft });
    };

    private renderFromAccount(): React.ReactNode {
        const currentDraft = this.state.currentDraft;

        let inner;
        if (this.state.accounts) {
            let selectedAccountKey = undefined;
            const options = Object.values(this.state.accounts)
                .map(({ entity, account }) => {
                    const key = entity.id + account.id;
                    if (currentDraft.account?.entityId == entity.id &&
                        currentDraft.account?.traitId == account.id) {
                        selectedAccountKey = key;
                    }

                    return <option value={key} key={key}>{account.message.name}</option>;
                });

            inner = (
                <select value={selectedAccountKey} onChange={this.handleChangeAccount}>
                    {options}
                </select>
            );

        } else {
            inner = <span>Loading...</span>;
        }

        return (
            <div className="subject field">
                <span className="field-label">From</span>
                <span className="field-content">{inner}</span>
            </div>
        );
    }

    private handleChangeAccount = (e: ChangeEvent<HTMLSelectElement>): void => {
        const entityAccount = this.state.accounts[e.target.value];
        if (!entityAccount) {
            return;
        }

        const { entity, account } = entityAccount;

        const draft = this.state.currentDraft;
        draft.account = new exocore.store.Reference({
            entityId: entity.id,
            traitId: account.id,
        });
        this.setState({ currentDraft: draft });

        this.saveObject();
    };

    private renderContactField(fieldName: string, displayName: string): React.ReactNode {
        const draftRecord = this.state.currentDraft as Record<string, exomind.base.v1.IContact[]>;
        const fieldContacts = draftRecord[fieldName] ?? [];

        const classes = classNames({
            [fieldName]: true,
            field: true
        });
        return <div className={classes}>
            <span className="field-label">{displayName}</span>
            <span className="field-content">
                <input
                    type="text" value={EmailUtil.formatContacts(fieldContacts, true)}
                    onChange={(e) => this.handleContactFieldChange(fieldName, e)}
                    onBlur={(e) => this.handleContactFieldBlur(fieldName, e)}
                    placeholder="Type recipients" />
            </span>
        </div>;
    }

    private handleContactFieldChange(fieldName: string, e: ChangeEvent<HTMLInputElement>): void {
        const contactsString = e.target.value;

        const contacts = EmailUtil.parseContacts(contactsString);

        const lastChar = _.last(contactsString.trim());
        if (lastChar == ',') {
            contacts.push(new exomind.base.v1.Contact());
        } else if (lastChar == '<') {
            const lastContact = contacts[contacts.length - 1];
            lastContact.name = lastContact.email;
            lastContact.email = '';
        }

        const draftRecord = this.state.currentDraft as Record<string, exomind.base.v1.IContact[]>;
        draftRecord[fieldName] = contacts;

        this.setState({});
    }

    private handleContactFieldBlur(fieldName: string, e: ChangeEvent<HTMLInputElement>): void {
        const contacts = EmailUtil.parseContacts(e.target.value);
        const draft = this.state.currentDraft as Record<string, exomind.base.v1.IContact[]>;
        draft[fieldName] = contacts;

        this.saveObject();
    }

    private renderBody(): React.ReactNode {
        let editPart = EmailUtil.extractHtmlPart(this.state.currentDraft.parts);
        if (!editPart) {
            if (!_.isEmpty(this.state.currentDraft.parts)) {
                editPart = _.first(this.state.currentDraft.parts);
            } else {
                editPart = new exomind.base.v1.EmailPart({
                    mimeType: 'text/html'
                });
            }
        }

        return (
            <HtmlEditor
                content={editPart.body}
                onBound={this.handleHtmlEditorBound}
                onChange={(content) => this.handleBodyChange(editPart, content)}
                onBlur={this.saveObject} />
        );
    }

    private handleHtmlEditorBound = (editor: HtmlEditor): void => {
        this.setState({
            editor: editor
        });
    };

    private handleBodyChange(editPart: exomind.base.v1.IEmailPart, content: string): void {
        const newEditPart = new exomind.base.v1.EmailPart(editPart);
        newEditPart.body = content;

        const draft = this.state.currentDraft;
        draft.parts = [newEditPart];
        this.setState({ currentDraft: draft });

        // if after a second, it's still the same body, we save it (debouncing)
        setTimeout(() => {
            if (this.state.currentDraft.parts[0].body === content) {
                this.saveObject();
            }
        }, 1000);
    }

    private saveObject = (): void => {
        if (this.state && !_.isEqual(this.state.currentDraft, this.state.savedDraft)) {
            const mutation = MutationBuilder
                .updateEntity(this.props.entity.entity.id)
                .putTrait(this.state.currentDraft, this.props.draftTrait.id)
                .build();

            Exocore.store.mutate(mutation);

            if (this.mounted) {
                this.setState({
                    savedDraft: new exomind.base.v1.DraftEmail(this.state.currentDraft),
                });
            }
        }
    };
}
