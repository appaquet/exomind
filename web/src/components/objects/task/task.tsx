import { Exocore, MutationBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../store/entities';
import EditableText from '../../interaction/editable-text/editable-text.js';
import { Selection } from '../entity-list/selection';
import './task.less';

interface IProps {
    entity: EntityTraits;
    taskTrait: EntityTrait<exomind.base.ITask>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
}

interface IState {
    currentTask: exomind.base.ITask
}

export default class Task extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);

        this.state = {
            currentTask: new exomind.base.Task(props.taskTrait.message),
        }
    }

    render(): React.ReactNode {
        return (
            <div className="entity-component task">
                <div className="object-summary">
                    <div className="name field">
                        <span className="field-label">Name</span>
                        <span className="field-content">
                            <EditableText text={this.state.currentTask.title} onChange={this.handleNameChange} />
                        </span>
                    </div>
                </div>
            </div>
        );
    }

    private handleNameChange = (newTitle: string): void => {
        const task = this.state.currentTask;
        task.title = newTitle;

        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(task, this.props.taskTrait.id)
            .build();
        Exocore.store.mutate(mutation);

        this.setState({
            currentTask: task,
        });
    }

}

