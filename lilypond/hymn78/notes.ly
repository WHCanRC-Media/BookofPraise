melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 f4 c'4 a4 g8 f4 e4 d4 c4 d4 e4 f4 g2 f2 r4 \break

  % Line 2
  f4 f4 c'4 a4 g8 f4 e4 d4 c4 d4 e4 f4 g2 f2 r4 \break

  % Line 3
  c'4 c4 c4 d2 a4 bes4 c4 c4 bes4 a4 g2 \break

  % Line 4
  c,4 d4 e4 f4 g4 a4 g2 f2 \bar "|."
}
