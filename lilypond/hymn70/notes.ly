melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 a4 c4 c4 f,4 f4 a4 a4 \break

  % Line 2
  d,4 e4 f4 g4 a4 c4 g2 \break

  % Line 3
  a4 a4 c4 c4 f,4 f4 a4 a4 \break

  % Line 4
  d,4 e4 f4 bes4 g4 c4 a2 \break

  % Line 5
  a2 c2 d2 c2 a4 c4 f,4 bes4 a2 g4 f1 \bar "|."
}
