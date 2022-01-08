import { Exocore, MutationBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import EditableText from '../../interaction/editable-text/editable-text';
import { Selection } from '../entity-list/selection';
import './task.less';
import { ContainerState } from '../container-state';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import { observer } from 'mobx-react';
import classNames from 'classnames';
import _ from 'lodash';

interface IProps {
    entity: EntityTraits;
    taskTrait: EntityTrait<exomind.base.v1.ITask>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState,
}

interface IState {
    currentTask: exomind.base.v1.ITask
    focused: boolean;
}

@observer
export default class Task extends React.Component<IProps, IState> {
    private shortcutToken: ListenerToken;
    private input?: EditableText;

    constructor(props: IProps) {
        super(props);

        this.state = {
            currentTask: new exomind.base.v1.Task(props.taskTrait.message),
            focused: false,
        };

        this.shortcutToken = Shortcuts.register([
            {
                key: ['Enter'],
                callback: this.handleShortcutFocus,
                disabledContexts: ['input', 'modal'],
            },
        ], props.containerState?.active ?? false);
    }

    componentWillUnmount(): void {
        Shortcuts.unregister(this.shortcutToken);
    }

    componentDidUpdate(prevProps: IProps) {
        // allow incoming changes if we're not focused on task
        const prevTask = new exomind.base.v1.Task(prevProps.taskTrait.message);
        const newTask = new exomind.base.v1.Task(this.props.taskTrait.message);
        if (!this.state.focused && !_.isEqual(prevTask, newTask)) {
            this.setState({
                currentTask: newTask,
            });
        }

        Shortcuts.setListenerEnabled(this.shortcutToken, this.props.containerState?.active ?? false);
    }

    render(): React.ReactNode {
        const classes = classNames({
            'entity-component': true,
            'task': true,
            'active': this.props.containerState?.active ?? false,
        });

        return (
            <div className={classes}>
                <div className="entity-details">
                    <div className="name field">
                        <span className="field-label">Name</span>
                        <span className="field-content">
                            <EditableText
                                text={this.state.currentTask.title}
                                onChange={this.handleNameChange}
                                onBound={this.handleInputBound}
                                onFocus={this.handleFocus}
                                onBlur={this.handleBlur}
                            />
                        </span>
                    </div>
                </div>
            </div>
        );
    }


    private handleNameChange = (newTitle: string): void => {
        const task = this.state.currentTask;
        if (task.title === newTitle) {
            return;
        }

        task.title = newTitle;
        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(task, this.props.taskTrait.id)
            .build();
        Exocore.store.mutate(mutation);

        this.setState({
            currentTask: task,
        });
    };

    private handleInputBound = (editable: EditableText): void => {
        this.input = editable;
    };

    private handleShortcutFocus = (): boolean => {
        if (this.input) {
            this.input.focus();
            return true;
        } else {
            return false;
        }
    };

    private handleFocus = (): void => {
        if (!this.state.focused) {
            this.setState({
                focused: true,
            });
        }
    };

    private handleBlur = (): void => {
        if (this.state.focused) {
            this.setState({
                focused: false,
            });
        }
    };
}

