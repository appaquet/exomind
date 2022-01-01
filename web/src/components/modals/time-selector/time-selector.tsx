import classNames from 'classnames';
import React from 'react';
import Flatpickr from "react-flatpickr";
import "flatpickr/dist/themes/light.css";
import './time-selector.less';
import DateUtil, { ISnoozeChoice, SnoozeKey } from '../../../utils/dates';
import { ListenerToken, Shortcuts } from '../../../shortcuts';

interface IProps {
    onSelectionDone: (date: Date) => void;
}

interface IState {
    picker: boolean;
    date: Date;
    selected?: number;
}

const choicePerRow = 3;

export default class TimeSelector extends React.Component<IProps, IState> {
    private listElemRef: React.RefObject<HTMLUListElement> = React.createRef();
    private shortcutToken: ListenerToken;
    private choices: ISnoozeChoice[];

    constructor(props: IProps) {
        super(props);

        this.state = {
            picker: false,
            date: new Date(),
        };

        this.choices = DateUtil.getSnoozeChoices();

        this.shortcutToken = Shortcuts.register([
            {
                key: 'ArrowUp',
                callback: () => this.handleShortcutMove(-choicePerRow),
            },
            {
                key: 'ArrowDown',
                callback: () => this.handleShortcutMove(choicePerRow),
            },
            {
                key: 'ArrowLeft',
                callback: () => this.handleShortcutMove(-1),
            },
            {
                key: 'ArrowRight',
                callback: () => this.handleShortcutMove(1),
            },
            {
                key: 'Enter',
                callback: this.handleShortcutSelect,
            },
        ]);
    }

    componentWillUnmount(): void {
        Shortcuts.unregister(this.shortcutToken);
    }

    render(): React.ReactNode {
        const body = (!this.state.picker) ? this.renderList() : this.renderPicker();
        const classes = classNames({
            'time-selector': true,
            choices: !this.state.picker,
            picker: !!this.state.picker
        });

        return (
            <div className={classes}>
                <div className="time-selector-header">Snooze until...</div>
                {body}
            </div>
        );
    }

    private renderList(): React.ReactNode {
        return (
            <ul ref={this.listElemRef}>
                {this.renderLaterChoices()}
            </ul>
        );
    }

    private renderLaterChoices(): React.ReactNode {
        return this.choices
            .map((choice, i) => {
                const selected = this.state.selected === i;
                const classes = classNames({
                    selected
                });
                return (
                    <li className={classes} onClick={() => this.handleTimeClick(choice.key)} key={choice.key}>
                        <span>{choice.copy}</span>
                    </li>
                )
            });
    }


    private renderPicker(): React.ReactNode {
        return (
            <div>
                <div className="field">
                    <Flatpickr
                        data-enable-time
                        value={this.state.date}
                        onChange={this.handlePickerChange}
                    />
                </div>
                <div className="button">
                    <button onClick={this.handlePickerDone}>
                        <span>Done</span>
                    </button>
                </div>
            </div>
        );
    }

    private handlePickerChange = (dates: Date[]): void => {
        this.setState({ date: dates[0] });
    }

    private handlePickerDone = (): void => {
        if (this.state.date) {
            this.props.onSelectionDone(this.state.date);
        }
    }

    private handleTimeClick(key: SnoozeKey): void {
        if (key === 'pick') {
            this.setState({
                picker: true
            });
        } else {
            const date = DateUtil.snoozeDate(key);
            if (this.props.onSelectionDone) {
                this.props.onSelectionDone(date);
            }
        }
    }

    private handleShortcutMove(moveCount: number): boolean {
        const nbChoices = this.choices.length;

        let selected: number;
        if (this.state.selected !== undefined) {
            // wrap around
            selected = this.state.selected + moveCount;
            if (selected >= nbChoices) {
                selected -= nbChoices;
            } else if (selected < 0) {
                selected += nbChoices;
            }
        } else {
            selected = 0;
        }

        this.setState({ selected });

        return true;
    }

    private handleShortcutSelect = (): boolean => {
        if (this.state.picker && this.state.date) {
            this.props.onSelectionDone(this.state.date);
            return true;
        }

        if (this.state.selected !== undefined) {
            const choice = this.choices[this.state.selected];
            if (choice.key == 'pick') {
                this.setState({
                    picker: true
                });
            } else {
                const date = DateUtil.snoozeDate(choice.key);
                this.props.onSelectionDone?.(date);
            }
            return true;
        }

        return false;
    }
}

