melody = \relative c' {
  \clef treble
  \key ees \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  ees4 g4 aes4 bes4 ees,4 f4 g4 aes4 \break

  % Line 2
  g4 f4 ees4 ees4 d4 ees2 \break

  % Line 3
  bes'4 ees4 d4 c4 bes4 bes4 aes4 bes4 \break

  % Line 4
  g4 f4 ees4 ees4 d4 ees2 \bar "|."
}
