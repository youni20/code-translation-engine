# Bsc Thesis

### Research Topic

We are testing whether the quality of the feedback signal in a post-translation repair loop causally affects translation success, by isolating feedback type (raw compiler output vs structured LSP diagnostics) as the sole independent variable in an agentic code translation pipeline.

<br>
&nbsp;
<br>

### Flow Chart Of The Process
![Flow Chart of Process](images/cte_pipeline_diagram.png)

<br>
&nbsp;

- Condition A (Baseline / Control)
The repair agent receives the broken code plus the raw compiler stderr as feedback. This is how existing pipelines like in the paper provided by Softwerk already work. Just the standard error output you would see if you ran rustc in your terminal.

- Condition B (Treatment)
The repair agent receives the broken code plus the structured LSP diagnostics that have been processed through your Diagnostic Parser and Formatter. This is the new thing you are introducing.

<br>
&nbsp;
Everything else is held constant between the two conditions: the same model, the same starting translated code, the same prompt template structure, the same iteration cap, and the same validation step. The only variable is the feedback string.