import { Exocore, MutationBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import EditableText from '../../interaction/editable-text/editable-text';
import { Selection } from '../entity-list/selection';
import './task.less';

interface IProps {
    entity: EntityTraits;
    taskTrait: EntityTrait<exomind.base.v1.ITask>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
}

interface IState {
    currentTask: exomind.base.v1.ITask
}

export default class Task extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);

        this.state = {
            currentTask: new exomind.base.v1.Task(props.taskTrait.message),
        }
    }

    render(): React.ReactNode {
        return (
            <div className="entity-component task">
                <div className="entity-details">
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

