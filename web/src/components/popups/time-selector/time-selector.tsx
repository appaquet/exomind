import classNames from 'classnames';
import moment from 'moment';
import React from 'react';
import TimeLogic from '../../../logic/time-logic.js';
import './time-selector.less';

interface IProps {
    onSelectionDone: (date: Date) => void;
}
interface IState {
    picker: boolean;
    date: Date;
}

export default class TimeSelector extends React.Component<IProps, IState> {
    listElemRef: React.RefObject<HTMLUListElement>;

    constructor(props: IProps) {
        super(props);

        this.state = {
            picker: false,
            date: new Date(),
        };

        this.listElemRef = React.createRef();
    }

    render(): React.ReactNode {
        const body = (!this.state.picker) ? this.renderList() : this.renderPicker();
        const classes = classNames({
            'time-selector': true,
            choices: !this.state.picker,
            picker: !!this.state.picker
        });

        return (
            <div className={classes} onMouseOver={this.handlePreventDefault.bind(this)}>
                <div className="time-selector-header">Postpone until</div>
                {body}
            </div>
        );
    }

    private renderList(): React.ReactNode {
        return (
            <ul ref={this.listElemRef} onClick={this.handlePreventDefault.bind(this)} onWheel={this.handleListWheel.bind(this)}>
                {this.renderLaterChoices()}
            </ul>
        );
    }

    private renderLaterChoices(): React.ReactNode {
        return TimeLogic.getLaterChoices()
            .map((choice) => {
                return (
                    <li onClick={this.handleTimeClick.bind(this, choice.key)} key={choice.key}>
                        <span>{choice.copy}</span>
                    </li>
                )
            });
    }


    private renderPicker(): React.ReactNode {
        // eslint-disable-next-line @typescript-eslint/no-var-requires
        const DateTimeField = require('@blen/react-bootstrap-datetimepicker');
        return (
            <div>
                <div className="field">
                    <DateTimeField dateTime={moment(this.state.date).format('x')}
                        inputFormat='MMMM Do YYYY, h:mm:ss a'
                        onChange={this.handlePickerChange.bind(this)} />
                </div>
                <button className="done" onClick={this.handlePickerDone.bind(this)}>
                    <span>Done</span>
                </button>
            </div>

        );
    }

    private handlePreventDefault(e: MouseEvent): void {
        // prevent browser from bubbling the click and mouseover under the list
        e.stopPropagation();
    }

    private handlePickerChange(date: Date): void {
        this.setState({
            date: date
        });
    }

    private handlePickerDone(): void {
        let date;
        if (this.state.date) {
            date = this.state.date;
        } else {
            date = new Date();
        }

        if (this.props.onSelectionDone) {
            this.props.onSelectionDone(date);
        }
    }

    private handleTimeClick(key: string): void {
        if (key === 'pick') {
            this.setState({
                picker: true
            });
        } else {
            const date = TimeLogic.textDiffToDate(key);
            if (this.props.onSelectionDone) {
                this.props.onSelectionDone(date);
            }
        }
    }

    /**
     * We catch wheel events to prevent scrolling parent when we're getting at the end or at the beginning of the list overflow
     */
    private handleListWheel(e: MouseWheelEvent): void {
        const listElem = this.listElemRef.current;
        if (listElem.scrollTop === 0 && e.deltaY < 0) {
            // if we are at the top, prevent scrolling up
            e.preventDefault();

        } else if ((listElem.scrollTop + listElem.clientHeight) >= listElem.scrollHeight && e.deltaY > 0) {
            // if we are at the bottom, prevent scrolling down
            e.preventDefault();
        }
    }
}

