
# Timestitch design

There are two use-cases supported by this tool:

1. "Journal mode": Constructing formatted overviews of data by period, sourced from a collection of implicitely per-day files (i.e., files whose naming scheme encodes the timeline position of the data contained within.)
2. "Timeline mode": Constructing formatted overvies of data by period, sourced from a collection of explicitly annotated date-time data fields without any particulare naming scheme.

To support both of these, we want to organize the inputs and process them into a uniform intermediate representation. From this representation, we can generate the desired output.

