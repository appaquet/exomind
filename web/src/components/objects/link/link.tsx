
import { Exocore, MutationBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTrait, EntityTraits } from '../../../store/entities';
import EditableText from '../../interaction/editable-text/editable-text.js';
import { Selection } from '../entity-list/selection';
import './link.less';

interface IProps {
    entity: EntityTraits;
    linkTrait: EntityTrait<exomind.base.ILink>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
}

interface IState {
    currentLink: exomind.base.ILink
}

export default class Link extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);

        this.state = {
            currentLink: new exomind.base.Link(props.linkTrait.message),
        }
    }

    render(): React.ReactNode {
        return (
            <div className="entity-component link">
                <div className="object-summary">
                    <div className="title field">
                        <span className="field-label">Title</span>
                        <span className="field-content"><EditableText text={this.state.currentLink.title}
                            onChange={this.handleTitleChange.bind(this)} /></span>
                    </div>
                    <div className="url field"><span className="field-label">URL</span> <span
                        className="field-content">{this.state.currentLink.url}</span></div>
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

    private handleTitleChange(newTitle: string): void {
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
    }
}

