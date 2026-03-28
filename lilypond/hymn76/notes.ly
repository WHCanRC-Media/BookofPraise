melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4. a8 c4 a4 f4 g4 a2 \break

  % Line 2
  a4. a8 c4 a4 f4 g4 a2 \break

  % Line 3
  a4. a8 bes4 bes4 g4. g8 a2 \break

  % Line 4
  a4 b4 c4 f,4 e4 d4 c2 \break

  % Line 5
  e4. e8 g4 e4 f4 g4 a2 \break

  % Line 6
  a4. a8 c4 a4 bes4 c4 d2 \break

  % Line 7
  d4. d8 bes4 g4 c4. c8 a2 \break

  % Line 8
  bes4 d4 c4 f,4 a4 g4 f2 \bar "|."
}
