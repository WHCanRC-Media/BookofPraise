melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 f4 e4 f4 a4 g4 g4 f4 \break

  % Line 2
  f4 bes4 a4 f4 g4 a2 \break

  % Line 3
  a4 a4 bes4 c4 a4 f4 g4 a4 \break

  % Line 4
  a4 g4 f4 f4 e4 f2 \bar "|."
}
