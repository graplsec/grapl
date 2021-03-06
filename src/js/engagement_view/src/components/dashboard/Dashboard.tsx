import React, { useEffect, useState } from "react";
import Button from "@material-ui/core/Button";
import { useAsync } from "react-async-hook";
import { checkLogin } from "services/login/checkLoginService";
import { Link } from "react-router-dom";
import { dasboardStyles } from "./styles";
import GraplHeader from "../reusableComponents/graplHeader/GraplHeader";
import LoginNotification from "../reusableComponents/notifications/LoginNotification";
import { getNotebookUrl } from "services/sagemakerNotebook/notebookService";

const useStyles = dasboardStyles;

export default function Dashboard() {
    const asyncSagemakerUrl = useAsync(getNotebookUrl, []);
    const classes = useStyles();

    const [state, setState] = useState({
        loggedIn: true,
        renderedOnce: false,
    });

    useEffect(() => {
        if (state.renderedOnce) {
            return;
        }

        const interval = setInterval(async () => {
            await checkLogin().then((loggedIn) => {
                if (!loggedIn) {
                    console.warn("Logged out");
                }
                setState({
                    loggedIn: loggedIn || false,
                    renderedOnce: true,
                });
            });
        }, 2000);

        return () => {
            clearInterval(interval);
        };
    }, [state, setState]);

    console.log("state - loggedin", state.loggedIn);

    const loggedIn = state.loggedIn;

    const openSagemakerUrl = () => {
        if (asyncSagemakerUrl.loading || !asyncSagemakerUrl.result) {
            alert("sagemaker url not ready");
            return;
        }

        window.open(asyncSagemakerUrl.result);
    };

    return (
        <>
            <GraplHeader displayBtn={false} />

            <div className={classes.dashboard}>
                <section className={classes.navSec}>
                    <Link to="/engagements" className={classes.link}>
                        {" "}
                        Engagements{" "}
                    </Link>
                    <Link to="/plugins" className={classes.link}>
                        {" "}
                        Upload Plugin{" "}
                    </Link>
                    <Button
                        onClick={openSagemakerUrl}
                        className={classes.sagemaker}
                    >
                        {" "}
                        Engagement Notebook{" "}
                    </Button>
                </section>

                <section>
                    <div className={classes.loggedIn}>
                        {!loggedIn ? <LoginNotification /> : ""}
                    </div>
                </section>
            </div>
        </>
    );
}
