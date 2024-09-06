
import { Exocore, MutationBuilder } from 'exocore';
import { observer } from 'mobx-react';
import React from 'react';
import Navigation from '../../../navigation';
import { exomind } from '../../../protos';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import EditableText from '../../interaction/editable-text/editable-text';
import { ContainerState } from '../container-state';
import { Selection } from '../entity-list/selection';
import './link.less';

interface IProps {
    entity: EntityTraits;
    linkTrait: EntityTrait<exomind.base.v1.ILink>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState,
}

interface IState {
    currentLink: exomind.base.v1.ILink
}

@observer
export default class Link extends React.Component<IProps, IState> {
    private shortcutToken: ListenerToken;

    constructor(props: IProps) {
        super(props);

        this.state = {
            currentLink: new exomind.base.v1.Link(props.linkTrait.message),
        };

        this.shortcutToken = Shortcuts.register({
            key: 'Enter',
            callback: this.handleShortcutEnter,
            disabledContexts: ['input', 'modal'],
        });
    }

    componentWillUnmount(): void {
        Shortcuts.unregister(this.shortcutToken);
    }

    render(): React.ReactNode {
        Shortcuts.setListenerEnabled(this.shortcutToken, !this.props.containerState.closed);

        return (
            <div className="entity-component link">
                <div className="entity-details">
                    <div className="title field">
                        <span className="field-label">Title</span>
                        <span className="field-content">
                            <EditableText text={this.state.currentLink.title} onChange={this.handleTitleChange} />
                        </span>
                    </div>
                    <div className="url field">
                        <span className="field-label">URL</span>
                        <span className="field-content with-ellipsis">{this.state.currentLink.url}</span>
                    </div>
                </div>

                <div className="object-body">
                    {this.renderBody()}
                </div>
            </div>
        );
    }

    private renderBody(): React.ReactNode {
        return <div className="open"><a href={this.state.currentLink.url} target="_blank" rel="noreferrer">Open link</a></div>;
    }

    private handleTitleChange = (newTitle: string): void => {
        const link = this.state.currentLink;
        link.title = newTitle;

        this.setState({
            currentLink: link,
        });

        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(this.state.currentLink, this.props.linkTrait.id)
            .build();

        Exocore.store.mutate(mutation);

        this.setState({
            currentLink: this.state.currentLink,
        });
    };

    private handleShortcutEnter = (): boolean => {
        Navigation.navigateExternal(this.state.currentLink.url);
        return true;
    };
}

